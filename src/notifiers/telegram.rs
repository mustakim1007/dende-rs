use anyhow::Result;
use teloxide::{prelude::*, types::ParseMode}; // brings Requester
use teloxide::types::ChatId;
use log::{info,debug,error};

#[derive(Clone)]
pub struct TelegramSink {
    bot: Bot,
    chat_id: ChatId,
}

impl std::fmt::Debug for TelegramSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TelegramSink(chat_id={})", self.chat_id.0)
    }
}

impl TelegramSink {
    pub fn new(token: String, chat_id: i64) -> Self {
        let bot = Bot::new(token);
        // Optional: validate token once; ignore errors to avoid failing the whole app.
        let bot_clone = bot.clone();
        tokio::spawn(async move {
            match bot_clone.get_me().await {
                Ok(me) => debug!(
                    "Telegram running as @{} (id={}) for one job!",
                    me.user.username.as_deref().unwrap_or("unknown"),
                    me.user.id.0
                ),
                Err(e) => error!("Telegram getMe error: {e}"),
            }
        });
        Self { bot, chat_id: ChatId(chat_id) }
    }

    pub async fn send(&self, html: &str) -> Result<()> {
        info!("Sending notification from telegram..");
        self.bot.send_message(self.chat_id, html).parse_mode(ParseMode::Html).await?;
        debug!("Sent by telegram to UserId({})",&self.chat_id);
        Ok(())
    }
}