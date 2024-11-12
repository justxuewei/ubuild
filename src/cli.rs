use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Do not mount ssh directory ($HOME/.ssh) to container.
    #[arg(long)]
    pub disable_ssh: bool,

    /// Do not use sudo.
    #[arg(long)]
    pub disable_sudo: bool,

    /// Enable rust cache for Rust. If true, it will mount:
    /// - $HOME/.cargo/git
    /// - $HOME/.cargo/registry
    #[arg(long, short)]
    pub enable_rust_cache: Option<bool>,

    /// Mount base path, the default value is $HOME.
    #[arg(short, long)]
    pub base_path: Option<String>,

    /// Default user in image.
    #[arg(long, default_value = "nonroot")]
    pub user: String,

    /// Name of image.
    pub image: String,

    pub command: Vec<String>,
}
