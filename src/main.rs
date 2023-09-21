#![allow(deprecated)]

mod cli;
mod engine;

use clap::Parser;
use cli::Args;
use engine::{Docker, Engine};

fn main() {
    let args = Args::parse();

    let engine = Docker::new();
    if let Err(err) = engine.run(&args) {
        panic!("{}", err);
    }
}
