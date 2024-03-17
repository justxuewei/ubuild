use std::env::{current_dir, home_dir};
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use tokio::process::Command;

use super::new_command;
use crate::cli::Args;
use crate::engine::Engine;

pub struct Docker {
    id: Option<String>,
    args: Args,
}

impl Docker {
    pub fn new(args: Args) -> Self {
        Self { id: None, args }
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

    fn build_docker_run_command(&mut self) -> Result<Command> {
        let cdir = current_dir().context("get current dir")?;
        // TODO: This might not work on the Windows.
        let hdir = home_dir().context("get home dir")?;
        let mount_path = if let Some(path) = self.args.base_path.as_ref() {
            PathBuf::from(path)
        } else {
            hdir.clone()
        };

        // docker run -d \
        //  -v $HOME:$HOME \
        //  -v $HOME/.ssh:/home/$user/.ssh \
        //  -w $(pwd) \
        //  $image \
        //  bash -c "source /home/$IMAGEUSER/.bashrc && $cmd"
        let mut cmd = new_command("docker", !self.args.no_sudo);
        cmd.args(["run", "-d"]);
        if !self.args.no_ssh {
            let mut ssh_dir = hdir.clone();
            ssh_dir.push(".ssh");
            if !ssh_dir.exists() {
                return Err(anyhow!(
                    "{} not exists, please use --no-ssh.",
                    ssh_dir.display()
                ));
            }
            cmd.args([
                "-v",
                &format!(
                    "{}:/home/{}/.ssh:ro",
                    ssh_dir.display(),
                    self.args.image_user
                ),
            ]);
        }

        if self.args.rust_cache {
            let cargo_regsitry_dir = hdir.join(".cargo").join("registry");
            if cargo_regsitry_dir.exists() {
                cmd.args([
                    "-v",
                    &format!(
                        "{}:/home/{}/.cargo/registry",
                        cargo_regsitry_dir.display(),
                        self.args.image_user
                    ),
                ]);
            }

            let cargo_git_dir = hdir.join(".cargo").join("git");
            if cargo_git_dir.exists() {
                cmd.args([
                    "-v",
                    &format!(
                        "{}:/home/{}/.cargo/git",
                        cargo_git_dir.display(),
                        self.args.image_user
                    ),
                ]);
            }
        }
        let ctr_cmd = self.args.command.join(" ");
        cmd.args([
            // -v $hdir:$hdir
            "-v",
            &format!("{}:{}", mount_path.display(), mount_path.display()),
            // -w $cdir
            "-w",
            &format!("{}", cdir.display()),
            // image
            self.args.image.as_str(),
            // container command
            "bash",
            "-c",
            &format!(
                "source /home/{}/.bashrc && {}",
                self.args.image_user, ctr_cmd
            ),
        ]);

        Ok(cmd)
    }
}

#[async_trait]
impl Engine for Docker {
    async fn run(&mut self) -> Result<ExitStatus> {
        self.check().await.context("check")?;

        let mut cmd = self.build_docker_run_command()?;

        let output = cmd.output().await?;
        let stdout = String::from_utf8(output.stdout)?;
        let stdout = stdout.trim_end_matches('\n');
        self.id = Some(stdout.to_string());

        if !output.status.success() {
            return Err(anyhow!("failed to exec docker run: {}", stdout));
        }

        let mut cmd = new_command("docker", !self.args.no_sudo);

        // docker logs -f {container id}
        cmd.args(["logs", "-f", stdout]);

        let mut child = cmd.spawn()?;

        Ok(child.wait().await?)
    }

    async fn clear(&self) -> Result<()> {
        if let Some(id) = self.id.as_ref() {
            let mut cmd = new_command("docker", !self.args.no_sudo);
            cmd.args(["rm", "-f", id]);
            cmd.stdout(Stdio::null());

            let mut child = cmd.spawn()?;
            child.wait().await?;
        }
        Ok(())
    }
}
