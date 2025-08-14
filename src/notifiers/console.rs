use anyhow::Result;
use log::info;

#[derive(Clone, Debug)]
pub struct ConsoleSink {
    _tag: String,
}

impl ConsoleSink {
    pub fn new(_tag: String) -> Self { Self { _tag } }

    pub async fn send(&self, text: &str) -> Result<()> {
        info!("Sending notification from console..");
        println!("\n{text}\n");
        Ok(())
    }
}