use anyhow::Result;
use async_trait::async_trait;
use teloxide::requests::Requester;
use teloxide::types::ChatId;
use teloxide::Bot;

use super::Notifier;
use crate::config::NotifierConfig;

pub struct Telegram {
    bot: Bot,
    chat_id: ChatId,
}

impl Telegram {
    pub async fn new_notifier(config: &NotifierConfig) -> Box<dyn Notifier> {
        let bot = Bot::new(&config.secret);

        Box::new(Telegram {
            bot,
            chat_id: ChatId(config.chat_id),
        })
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
}
