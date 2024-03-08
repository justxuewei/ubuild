mod docker;
pub use docker::Docker;

use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;

#[async_trait]
pub trait Engine {
    async fn run(&mut self) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}

pub(crate) fn new_command(cmd: &str, sudo: bool) -> Command {
    if sudo {
        let mut command = Command::new("sudo");
        command.arg(cmd);
        return command;
    }
    Command::new(cmd)
}
