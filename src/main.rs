#![allow(deprecated)]

use clap::Parser;
use cli::Args;
use engine::{Docker, Engine};

mod cli;
mod engine;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut engine = Docker::new(args.clone());

    if let Err(err) = engine.run().await {
        panic!("failed to run: {:?}", err);
    }
    if let Err(err) = engine.clear().await {
        panic!("failed to clear: {:?}", err);
    }
}
