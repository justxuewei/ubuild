#![allow(deprecated)]

use std::io::Write;

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use env_logger::{Builder, Env};

mod cli;
use cli::Args;

mod config;

mod engine;
use engine::{Docker, Engine};

mod notifier;
use log::{debug, error, LevelFilter};
use notifier::{serverchan::ServerChan, telegram::Telegram, Notifier};

fn init_log() {
    Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}

async fn send_notification(notifier: &Option<Box<dyn Notifier>>, exit_code: i32) -> Result<()> {
    let notifier = match notifier.as_ref() {
        Some(n) => n,
        None => {
            debug!("notifiction can't be sent due to no notifier");
            return Ok(());
        }
    };
    let now = Local::now();
    let formatted = now.format("%H:%M:%S").to_string();

    let message = if exit_code == 0 {
        format!("completed!\n{}", formatted)
    } else {
        format!("exited with non-zero code {}!\n{}", exit_code, formatted)
    };

    notifier.send("ubuild", &message).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    init_log();

    let args = Args::parse();
    let config = config::load().await;

    let mut engine = Docker::new(args.clone());

    let mut notifier = None;
    if let Some(nconfig) = config.notifier {
        match nconfig.r#type.to_lowercase().as_str() {
            "serverchan" => {
                notifier = Some(ServerChan::new_notifer(&nconfig).await);
            }
            "telegram" => {
                notifier = Some(Telegram::new_notifier(&nconfig).await);
            }
            _ => panic!("unknown notifier type: {}", nconfig.r#type),
        }
    }

    engine.run().await.unwrap();
    let exit_code = engine.exit_code().await;

    if let Err(err) = send_notification(&notifier, exit_code).await {
        error!("failed to send notification: {}", err);
    }

    if let Err(err) = engine.clear().await {
        panic!("failed to clear: {:?}", err);
    }
}
