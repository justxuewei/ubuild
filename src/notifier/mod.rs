use anyhow::Result;
use async_trait::async_trait;

pub mod serverchan;
pub mod telegram;

#[async_trait]
pub trait Notifier {
    async fn send(&self, title: &str, content: &str) -> Result<()>;
    async fn should_send(&self, elapsed: u64) -> bool;
}
