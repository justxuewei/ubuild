use anyhow::Result;
use async_trait::async_trait;

pub mod serverchan;

#[async_trait]
pub trait Notifier {
    async fn send(&self, title: &str, content: &str) -> Result<()>;
}
