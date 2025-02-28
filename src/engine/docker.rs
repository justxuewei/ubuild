use std::collections::HashMap;
use std::env::{current_dir, home_dir};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::{env, fs, io};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use log::{error, info};
use tokio::process::Command;

use crate::cli::Args;
use crate::config::Config;
use crate::engine::{new_command, Engine, HTTPS_PROXY, HTTP_PROXY};

const ROOT_PATH: &str = "/";

pub struct DockerConfig {
    disable_ssh: bool,
    disable_sudo: bool,
    enable_rust_cache: bool,
    base_path: Option<String>,
    user: String,
    image: String,
    command: Vec<String>,
}

impl DockerConfig {
    pub fn new(args: &Args, config: &Config) -> Self {
        let engine_config = config.engine.as_ref();
        Self {
            disable_ssh: args.disable_ssh,
            disable_sudo: args.disable_sudo,
            enable_rust_cache: args.enable_rust_cache.unwrap_or_else(|| {
                engine_config
                    .and_then(|c| c.enable_rust_cache)
                    .unwrap_or_default()
            }),
            base_path: args.base_path.clone(),
            user: args.user.clone(),
            image: args.image.clone(),
            command: args.command.clone(),
        }
    }
}

pub struct Docker {
    id: Option<String>,
    config: DockerConfig,
    exit_code: i32,
}

impl Docker {
    pub fn new(args: &Args, config: &Config) -> Self {
        Self {
            id: None,
            config: DockerConfig::new(args, config),
            exit_code: 0,
        }
    }
}

impl Docker {
    async fn check(&self) -> Result<()> {
        Command::new("docker")
            .output()
            .await
            .context("no docker installed")?;
        Ok(())
    }

    fn get_root_path(&self) -> Result<PathBuf> {
        let mut root_dir = current_dir()
            .context("get current dir")?
            .canonicalize()
            .context("canonicalize")?;

        if !has_read_permission(root_dir.as_path()).context("has read permission")? {
            return Err(anyhow!("no permission to read {:?}", root_dir));
        }

        // find root dir recursively, stop while no parent dir exists or no
        // permission.
        while let Some(parent) = root_dir.parent() {
            let parent_dir = parent.canonicalize().context("canonicalize")?;
            let parent_dir_name = parent_dir
                .as_path()
                .as_os_str()
                .to_str()
                .ok_or(anyhow!("empty parent dir path"))?;

            if parent_dir_name == ROOT_PATH
                || !has_read_permission(parent_dir.as_path()).context("has read permission")?
            {
                break;
            }
            let metadata = fs::metadata(parent).context("metadata")?;
            let mode = metadata.permissions().mode();
            if mode & 0o444 == 0 {
                break;
            }
            root_dir = parent.canonicalize().context("canonicalize")?;
        }

        Ok(root_dir)
    }

    fn build_docker_run_command(&mut self) -> Result<Command> {
        let current_dir = current_dir().context("get current dir")?;
        // TODO: This might not work on the Windows.
        let home_dir = home_dir().context("get home dir")?;
        let mount_path = if let Some(path) = self.config.base_path.as_ref() {
            PathBuf::from(path)
        } else {
            self.get_root_path().context("go to root")?
        };

        // docker run -d \
        //  -v $HOME:$HOME \
        //  -v $HOME/.ssh:/home/$user/.ssh \
        //  -w $(pwd) \
        //  $image \
        //  bash -c "source /home/$IMAGEUSER/.bashrc && $cmd"
        let mut cmd = new_command("docker", !self.config.disable_sudo);
        cmd.args(["run", "-d"]);
        if !self.config.disable_ssh {
            let mut ssh_dir = home_dir.clone();
            ssh_dir.push(".ssh");
            if !ssh_dir.exists() {
                return Err(anyhow!(
                    "{} not exists, please use --no-ssh.",
                    ssh_dir.display()
                ));
            }
            cmd.args([
                "-v",
                &format!("{}:/home/{}/.ssh:ro", ssh_dir.display(), self.config.user),
            ]);
        }

        if self.config.enable_rust_cache {
            let cargo_regsitry_dir = home_dir.join(".cargo").join("registry");
            if cargo_regsitry_dir.exists() {
                cmd.args([
                    "-v",
                    &format!(
                        "{}:/home/{}/.cargo/registry",
                        cargo_regsitry_dir.display(),
                        self.config.user
                    ),
                ]);
            }

            let cargo_git_dir = home_dir.join(".cargo").join("git");
            if cargo_git_dir.exists() {
                cmd.args([
                    "-v",
                    &format!(
                        "{}:/home/{}/.cargo/git",
                        cargo_git_dir.display(),
                        self.config.user
                    ),
                ]);
            }
        }

        let mut proxies = HashMap::new();
        if let Ok(v) = env::var(HTTP_PROXY) {
            proxies.insert(HTTP_PROXY.to_string(), v);
        }
        if let Ok(v) = env::var(HTTPS_PROXY) {
            proxies.insert(HTTPS_PROXY.to_string(), v);
        }
        // use host network to prevent from failure of connecting local proxies
        if !proxies.is_empty() {
            info!("found proxy settings, use host network");
            cmd.args(["--network", "host"]);
        }
        // set a series of proxy settings, e.g.
        // --env http_proxy=http://127.0.0.1:1088
        for (k, v) in proxies.iter() {
            cmd.args(["--env", &format!("{}={}", k, v)]);
        }

        let ctr_cmd = self.config.command.join(" ");
        cmd.args([
            // -v $hdir:$hdir
            "-v",
            &format!("{}:{}", mount_path.display(), mount_path.display()),
            // -w $cdir
            "-w",
            &format!("{}", current_dir.display()),
            // image
            self.config.image.as_str(),
            // container command
            "bash",
            "-c",
            &format!("source /home/{}/.bashrc && {}", self.config.user, ctr_cmd),
        ]);

        Ok(cmd)
    }
}

#[async_trait]
impl Engine for Docker {
    async fn run(&mut self) -> Result<()> {
        self.check().await.context("check")?;

        let mut cmd = self.build_docker_run_command()?;

        let output = cmd.output().await?;
        let stdout = String::from_utf8(output.stdout)?;
        let stdout = stdout.trim_end_matches('\n');
        self.id = Some(stdout.to_string());

        if !output.status.success() {
            return Err(anyhow!("failed to exec docker run: {}", stdout));
        }

        let mut cmd = new_command("docker", !self.config.disable_sudo);

        // docker logs -f {container id}
        cmd.args(["logs", "-f", stdout]);

        let mut child = cmd.spawn()?;

        self.exit_code = child.wait().await?.code().unwrap_or(0);

        Ok(())
    }

    async fn exit_code(&self) -> i32 {
        if self.exit_code != 0 {
            return self.exit_code;
        }

        let mut cmd = new_command("docker", !self.config.disable_sudo);
        cmd.args([
            "inspect",
            "-f",
            "{{.State.ExitCode}}",
            self.id.as_ref().unwrap(),
        ]);
        let stdout = match cmd.output().await {
            Ok(output) => output.stdout,
            Err(err) => {
                error!("failed to exec docker inspect: {}", err);
                return -1;
            }
        };
        let stdout = match String::from_utf8(stdout) {
            Ok(stdout) => stdout,
            Err(err) => {
                error!("failed to build string from output: {}", err);
                return -1;
            }
        };
        let stdout = stdout.trim_end_matches('\n');

        let exit_code: i32 = match stdout.parse() {
            Ok(code) => code,
            Err(err) => {
                error!("failed to parse exit code from \"{}\": {}", stdout, err);
                return -1;
            }
        };

        exit_code
    }

    async fn clear(&self) -> Result<()> {
        if let Some(id) = self.id.as_ref() {
            let mut cmd = new_command("docker", !self.config.disable_sudo);
            cmd.args(["rm", "-f", id]);
            cmd.stdout(Stdio::null());

            let mut child = cmd.spawn()?;
            child.wait().await?;
        }
        Ok(())
    }
}

fn has_read_permission(path: &Path) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_file() {
                match fs::File::open(path) {
                    Ok(_) => Ok(true),
                    Err(err) if err.kind() == io::ErrorKind::PermissionDenied => Ok(false),
                    Err(err) => Err(anyhow!("failed to open file: {:?}", err)),
                }
            } else if metadata.is_dir() {
                match fs::read_dir(path) {
                    Ok(_) => Ok(true),
                    Err(err) if err.kind() == io::ErrorKind::PermissionDenied => Ok(false),
                    Err(err) => Err(anyhow!("failed to read dir: {:?}", err)),
                }
            } else {
                Ok(false)
            }
        }
        Err(err) if err.kind() == io::ErrorKind::PermissionDenied => Ok(false),
        Err(err) => Err(anyhow!("failed to get metadata: {:?}", err)),
    }
}
