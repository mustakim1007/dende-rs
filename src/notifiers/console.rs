use anyhow::Result;
use log::info;

#[derive(Clone, Debug)]
pub struct ConsoleSink {
    tag: String,
}

impl ConsoleSink {
    pub fn new(tag: String) -> Self { Self { tag } }

    pub async fn send(&self, text: &str) -> Result<()> {
        info!("Sending notification from console..");
        info!("[NOTIFY -> {}] {}", self.tag, text);
        Ok(())
    }
}