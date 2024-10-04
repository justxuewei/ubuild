use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;

mod docker;
pub use docker::Docker;

pub(crate) const HTTP_PROXY: &str = "http_proxy";
pub(crate) const HTTPS_PROXY: &str = "https_proxy";

#[async_trait]
pub trait Engine {
    async fn run(&mut self) -> Result<()>;
    async fn exit_code(&self) -> i32;
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
