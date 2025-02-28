use std::{env, path::Path};

use tokio::fs;

const DEFAULT_CONFIG_PATH: &str = ".ubuild";

#[derive(serde::Deserialize, Default, Debug, Clone)]
pub struct Config {
    pub notifier: Option<NotifierConfig>,
    pub proxy: Option<ProxyConfig>,
    pub engine: Option<EngineConfig>,
}

#[derive(serde::Deserialize, Default, Debug, Clone)]
pub struct EngineConfig {
    pub enable_rust_cache: Option<bool>,
    pub base_path: Option<String>,
}

#[derive(serde::Deserialize, Default, Debug, Clone)]
pub struct NotifierConfig {
    pub r#type: String,
    pub secret: String,
    // If actual building time is less than this value, don't send
    // notification. The unit is in seconds.
    pub elapsed_threshold: u64,
    // Reserved for telegram
    pub chat_id: i64,
}

#[derive(serde::Deserialize, Default, Debug, Clone)]
pub struct ProxyConfig {
    pub http: Option<String>,
    pub https: Option<String>,
    pub sock5: Option<String>,
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
