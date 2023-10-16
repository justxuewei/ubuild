use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Do not mount ssh directory ($HOME/.ssh) to container.
    #[arg(long)]
    pub no_ssh: bool,

    /// Do not use sudo.
    #[arg(long)]
    pub no_sudo: bool,

    /// Enable cargo cache for Rust. The ubuild checks if
    /// "$HOME/.cargo/registry" exists. If true, mount this to container.
    #[arg(long, short)]
    pub cargo_cache: bool,

    /// Mount base path, the default value is $HOME.
    #[arg(short, long)]
    pub base_path: Option<String>,

    /// Default user in image.
    #[arg(long, default_value = "nonroot")]
    pub image_user: String,

    /// Name of image.
    pub image: String,

    pub command: Vec<String>,
}
