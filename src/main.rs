#![allow(deprecated)]

use clap::Parser;
use cli::Args;
use engine::{Docker, Engine};
use notifier::{serverchan::ServerChan, Notifier};

mod cli;
mod config;
mod engine;
mod notifier;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = config::load().await;

    let mut engine = Docker::new(args.clone());

    let notifier =
        config
            .notifier
            .as_ref()
            .map(|nconfig| match nconfig.r#type.to_lowercase().as_str() {
                "serverchan" => ServerChan::new(nconfig),
                _ => panic!("unknown notifier type: {}", nconfig.r#type),
            });

    let exitcode = engine.run().await.unwrap();

    if exitcode.success() {
        if let Some(notifier) = notifier {
            notifier.send("ubuild", "build completed!").await.unwrap();
        }
    }

    if let Err(err) = engine.clear().await {
        panic!("failed to clear: {:?}", err);
    }
}
