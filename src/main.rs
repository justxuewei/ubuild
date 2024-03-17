#![allow(deprecated)]

use std::io::Write;

use chrono::Local;
use clap::Parser;
use env_logger::{Builder, Env};

mod cli;
use cli::Args;

mod config;

mod engine;
use engine::{Docker, Engine};

mod notifier;
use log::{debug, LevelFilter};
use notifier::{serverchan::ServerChan, telegram::Telegram};

fn init_log() {
    Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
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

    let exitcode = engine.run().await.unwrap();

    if exitcode.success() {
        if let Some(notifier) = notifier {
            let now = Local::now();
            let formatted = now.format("%H:%M:%S").to_string();
            let ret = notifier
                .send("ubuild", &format!("build completed at {}!", formatted))
                .await;
            if let Err(err) = ret {
                println!("sent notification failed: {:?}", err);
            }
        } else {
            debug!("notifier is empty");
        }
    } else {
        debug!("engine.run() failed");
    }

    if let Err(err) = engine.clear().await {
        panic!("failed to clear: {:?}", err);
    }
}
