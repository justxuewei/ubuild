#![allow(deprecated)]

use chrono::Local;
use clap::Parser;

mod cli;
use cli::Args;

mod config;

mod engine;
use engine::{Docker, Engine};

mod notifier;
use notifier::{serverchan::ServerChan, Notifier};

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
            let now = Local::now();
            let formatted = now.format("%H:%M:%S").to_string();
            notifier
                .send("ubuild", &format!("build completed at {}!", formatted))
                .await
                .unwrap();
        }
    }

    if let Err(err) = engine.clear().await {
        panic!("failed to clear: {:?}", err);
    }
}
