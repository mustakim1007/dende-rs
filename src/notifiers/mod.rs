use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::task::JoinHandle;

pub mod telegram;
pub mod console;
// pub mod newnotifier;

use console::ConsoleSink;
use telegram::TelegramSink;
use log::error;

use crate::utils::date::timestamp;

/// Aggregates all selected sinks and dispatches notifications to them.
pub struct Notifier {
    tx: tokio::sync::mpsc::UnboundedSender<NotifyEvent>,
    #[allow(dead_code)]
    task: JoinHandle<()>,
}

#[derive(Clone, Debug)]
pub struct NotifyEvent {
    pub file: PathBuf,
    pub line_no: u64,
    pub line: String,
}

/// Concrete sink types we support. Add new variants as you add files.
enum Sink {
    Console(ConsoleSink),
    Telegram(TelegramSink),
    // NewNotifier(NewNotifierSink),
}

impl Sink {
    async fn send(&self, _text: &str, _text_no_content: &str, _html: &str) -> Result<()> {
        match self {
            Sink::Console(s) => s.send(_text_no_content).await,
            Sink::Telegram(s) => s.send(_html).await,
            // // Easy to add another notifier here
            // Sink::Telegram(s) => s.send(_text).await,
        }
    }
}

impl Notifier {
    /// Build sinks from `dest` strings. Telegram entries look like `tg:<CHAT_ID>`.
    pub fn new(dests_raw: Vec<String>, telegram_token: Option<String>) -> Result<Self> {
        let mut sinks: Vec<Sink> = Vec::new();

        for d in dests_raw {
            if let Some(rest) = d.strip_prefix("tg:") {
                match rest.parse::<i64>() {
                    Ok(id) => {
                        if let Some(token) = telegram_token.clone() {
                            sinks.push(Sink::Telegram(TelegramSink::new(token, id)));
                        } else {
                            error!("Skipping Telegram dest {rest}: no token provided");
                            sinks.push(Sink::Console(ConsoleSink::new(format!(
                                "INVALID_TG_NO_TOKEN({rest})"
                            ))));
                        }
                    }
                    Err(_) => sinks.push(Sink::Console(ConsoleSink::new(format!(
                        "INVALID_TG_ID({rest})"
                    )))),
                }
            } else {
                sinks.push(Sink::Console(ConsoleSink::new(d)));
            }
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<NotifyEvent>();

        let task = tokio::spawn(async move {
            while let Some(ev) = rx.recv().await {
                let text = format!(
                    "[logwatch] {}: line {} matched:
{}",
                    ev.file.display(),
                    ev.line_no,
                    ev.line.trim_end()
                );
                
                let text_no_content = format!(
                    "[logwatch] {}: line {} matched",
                    ev.file.display(),
                    ev.line_no,
                );

                let html = format!(
                    "<b>MATCHED</b>\n\n<i>Date:</i> <b>{}</b>\n<i>Filename and line:</i> <b>{}:{}</b>\n<i>Content matched:</i>\n\n{}",
                    timestamp(),
                    ev.file.display(),
                    ev.line_no,
                    ev.line.trim_end()
                );

                for sink in &sinks {
                    if let Err(e) = sink.send(&text, &text_no_content, &html).await {
                        error!("notifier sink error: {e}");
                    }
                }
            }
        });

        Ok(Self { tx, task })
    }

    /// Queue a notification event for processing by the async task.
    pub fn notify(&self, file: &Path, line_no: u64, line: &str) {
        let _ = self.tx.send(NotifyEvent {
            file: file.to_path_buf(),
            line_no,
            line: line.to_string(),
        });
    }
}