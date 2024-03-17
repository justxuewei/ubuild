use std::{env, path::Path};

use tokio::fs;

const DEFAULT_CONFIG_PATH: &str = ".ubuild";

#[derive(serde::Deserialize, Default, Debug)]
pub struct Config {
    pub notifier: Option<NotifierConfig>,
}

#[derive(serde::Deserialize, Default, Debug)]
pub struct NotifierConfig {
    pub r#type: String,
    pub secret: String,
    // reserved for telegram
    pub chat_id: i64,
}

pub async fn load() -> Config {
    // TODO: Custom config path
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("/"));
    let config_path = Path::new(&home_dir).join(DEFAULT_CONFIG_PATH);

    let config_literal = match fs::read_to_string(config_path).await {
        Ok(config) => config,
        Err(_) => return Config::default(),
    };

    toml::from_str(&config_literal).unwrap_or_default()
}
