use anyhow::Result;
use async_trait::async_trait;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};

use crate::config::NotifierConfig;

use super::Notifier;

pub struct ServerChan {
    send_key: String,
}

impl ServerChan {
    pub async fn new_notifer(config: &NotifierConfig) -> Box<dyn Notifier> {
        Box::new(ServerChan {
            send_key: config.secret.clone(),
        })
    }
}

#[async_trait]
impl Notifier for ServerChan {
    async fn send(&self, title: &str, desp: &str) -> Result<()> {
        let params = [("text", title), ("desp", desp)];
        let post_data = serde_urlencoded::to_string(params)?;
        let url = format!("https://sctapi.ftqq.com/{}.send", self.send_key);
        let client = reqwest::Client::new();
        let res = client
            .post(&url)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(CONTENT_LENGTH, post_data.len() as u64)
            .body(post_data)
            .send()
            .await?;
        if res.status() != 200 {
            println!("WARN: serverchan send failed: {}", res.text().await?);
        }

        Ok(())
    }
}
