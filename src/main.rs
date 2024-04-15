#![allow(deprecated)]

use std::{env, io::Write};

mod cli;
use cli::Args;
mod config;
mod engine;
use engine::{Docker, Engine};
mod notifier;
use notifier::serverchan::ServerChan;
use notifier::telegram::Telegram;
use notifier::Notifier;

use anyhow::{anyhow, Context, Result};
use chrono::Local;
use clap::Parser;
use config::Config;
use env_logger::{Builder, Env};
use log::{debug, error, info};

fn init_log() {
    let mut builder =
        Builder::from_env(Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
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

async fn new_notifier(config: &Config) -> Result<Option<Box<dyn Notifier>>> {
    if let Some(nconfig) = config.notifier.as_ref() {
        match nconfig.r#type.to_lowercase().as_str() {
            "serverchan" => Ok(Some(
                ServerChan::new_notifer(config)
                    .await
                    .context("new serverchan notifier")?,
            )),
            "telegram" => Ok(Some(
                Telegram::new_notifier(config)
                    .await
                    .context("new telegram notifier")?,
            )),
            _ => Err(anyhow!("unknown notifier type")),
        }
    } else {
        Ok(None)
    }
}

async fn setup_proxy(config: &Config) -> Result<()> {
    let proxy_config = match config.proxy.as_ref() {
        Some(pc) => pc,
        None => return Ok(()),
    };

    if let Some(http) = proxy_config.http.as_ref() {
        debug!("set http_proxy to {}", http);
        env::set_var("http_proxy", http);
    }

    if let Some(https) = proxy_config.https.as_ref() {
        debug!("set https_proxy to {}", https);
        env::set_var("https_proxy", https);
    }

    if let Some(sock5) = proxy_config.sock5.as_ref() {
        debug!("set all_proxy to {}", sock5);
        env::set_var("all_proxy", sock5);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    init_log();

    let args = Args::parse();
    let config = config::load().await;

    setup_proxy(&config).await.unwrap();
    let mut engine = Docker::new(args.clone());
    let notifier = new_notifier(&config).await.unwrap();

    engine.run().await.unwrap();
    let exit_code = engine.exit_code().await;
    info!(">>>>> task exited with code {}", exit_code);

    if let Err(err) = send_notification(&notifier, exit_code).await {
        error!("failed to send notification: {}", err);
    }

    if let Err(err) = engine.clear().await {
        panic!("failed to clear: {:?}", err);
    }
}
