use anyhow::{anyhow, Result};
use async_trait::async_trait;
use teloxide::requests::Requester;
use teloxide::types::ChatId;
use teloxide::Bot;

use super::Notifier;
use crate::config::Config;

pub struct Telegram {
    bot: Bot,
    chat_id: ChatId,
    elapsed_threshold: u64,
}

impl Telegram {
    pub async fn new_notifier(config: &Config) -> Result<Box<dyn Notifier>> {
        let notifier_config = match config.notifier.as_ref() {
            Some(nc) => nc,
            None => return Err(anyhow!("no notifier config")),
        };

        let bot = Bot::new(&notifier_config.secret);

        Ok(Box::new(Telegram {
            bot,
            chat_id: ChatId(notifier_config.chat_id),
            elapsed_threshold: notifier_config.elapsed_threshold,
        }))
    }
}

#[async_trait]
impl Notifier for Telegram {
    async fn send(&self, title: &str, desp: &str) -> Result<()> {
        self.bot
            .send_message(self.chat_id, format!("{}: {}", title, desp))
            .await?;

        Ok(())
    }

    async fn should_send(&self, elapsed: u64) -> bool {
        elapsed >= self.elapsed_threshold
    }
}
