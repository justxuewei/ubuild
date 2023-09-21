mod docker;
pub use docker::Docker;

use anyhow::Result;

use crate::cli::Args;

pub trait Engine {
    fn check(&self) -> Result<()>;
    fn run(&self, args: &Args) -> Result<()>;
}
