use std::env::{current_dir, home_dir};
use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::cli::Args;
use crate::engine::Engine;

pub struct Docker {}

impl Docker {
    pub fn new() -> Self {
        Self {}
    }
}

impl Engine for Docker {
    fn check(&self) -> Result<()> {
        // Check if docker was installed
        Command::new("docker")
            .output()
            .context("no docker installed")?;
        Ok(())
    }

    fn run(&self, args: &Args) -> Result<()> {
        self.check().context("check")?;

        let cdir = current_dir().context("get current dir")?;
        // TODO: This might not work on the Windows.
        let hdir = home_dir().context("get home dir")?;
        let mount_path = if let Some(path) = args.base_path.as_ref() {
            PathBuf::from(path)
        } else {
            hdir.clone()
        };

        // docker run --rm \
        //  -v $HOME:$HOME \
        //  -v $HOME/.ssh:/home/$user/.ssh \
        //  -w $(pwd) \
        //  $image \
        //  bash -c "source /home/$IMAGEUSER/.bashrc && $cmd"
        let mut cmd = if args.no_sudo {
            Command::new("docker")
        } else {
            let mut cmd = Command::new("sudo");
            cmd.arg("docker");
            cmd
        };
        cmd.args(["run", "--rm"]);
        if !args.no_ssh {
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
                &format!("{}:/home/{}/.ssh:ro", ssh_dir.display(), args.image_user),
            ]);
        }

        if args.cargo_cache {
            let cargo_cache_dir = hdir.join(".cargo").join("registry");
            if cargo_cache_dir.exists() {
                cmd.args([
                    "-v",
                    &format!(
                        "{}:/home/{}/.cargo/registry",
                        cargo_cache_dir.display(),
                        args.image_user
                    ),
                ]);
            }
        }
        let ctr_cmd = args.command.join(" ");
        cmd.args([
            // -v $hdir:$hdir
            "-v",
            &format!("{}:{}", mount_path.display(), mount_path.display()),
            // -w $cdir
            "-w",
            &format!("{}", cdir.display()),
            // image
            args.image.as_str(),
            // container command
            "bash",
            "-c",
            &format!("source /home/{}/.bashrc && {}", args.image_user, ctr_cmd),
        ]);

        let mut child = cmd.spawn().context("docker run")?;
        child.wait().context("wait child")?;

        Ok(())
    }
}
