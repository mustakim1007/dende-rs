<hr />

- [Usage](#usage)
- [Quick start](#quick-start)
- [YAML example](#yaml-example)
- [How to add a new notifier?](#how-to-add-a-new-notifier)

<hr />

## Usage

```bash
Monitor logs files and folders and notify from Telegram bot when a search or regex matches!

Usage: dende-rs [OPTIONS]

Options:
  -P, --path <PATH>
          Path to watch (file or folder, single-job CLI mode)
  -S, --search <SEARCH>
          Literal term to search for (single-job CLI mode)
  -R, --regex <REGEX>
          Regular expression (Rust regex, single-job CLI mode))
  -T, --to <TO>
          Recipients: plain text = console tag, 'tg:<CHAT_ID>' = Telegram (single-job CLI mode)
      --recursive
          Watch subdirectories (single-job CLI mode)
      --read-existing
          On startup, read existing files from the beginning (single-job CLI mode)
      --telegram-token <TELEGRAM_TOKEN>
          Telegram bot token (or ENV TELEGRAM_BOT_TOKEN) (single-job CLI mode) [env: TELEGRAM_BOT_TOKEN=]
  -C, --config <CONFIG>
          YAML configuration file (multi-jobs)
  -v...
          Verbosity (-v, -vv, -vvv)
  -h, --help
          Print help
```

## Quick Start

1. Download [Rust](https://www.rust-lang.org/tools/install).
2. Create a new bot using [@Botfather](https://core.telegram.org/bots/tutorial#obtain-your-bot-token) to get a token in the format `0123456789:XXXXxXXxxxXxX3x3x-3X-XxxxX3XXXXxx3X`.
3. Put these token into YAML file or directly as CLI argument.
4. Obtain your Telegram UserId for user who would like to contact, by sending `/start` to [@userinfo](https://telegram.me/userinfobot).
5. Put these UserId YAML file like `tg:UserId` or directly as CLI argument.
6. Please have each Telegram user send /start to your bot (bots can’t initiate DMs).
7. Make sure that your [Rust](https://www.rust-lang.org/tools/install) compiler is up to date:

```bash
$ rustup update nightly
$ rustup override set nightly
```

8. Compilation:

```bash
cargo build --release
```

9. Usage:

```bash
# YAML multi-job mode
target/debug/dende --config ./config_example.yaml

# Single-job CLI from string search
target/release/dende-rs -- \
  -P /var/log/myapp \
  -S ERROR \
  -T console -T tg:123456789 \
  --recursive

# Single-job CLI from regex search
target/release/dende-rs -P /var/log/myapp/access.log -R "^SUCCESS.*" -T tg:123456789
```

## YAML example

```yaml
# config.yaml
telegram_token: "1234567890:FIXME-FIXME"  # Telegram API token for you bot

jobs:
  # Job 1
  - path: "/tmp/logs/apache2/"            # Path of main folder where to search
    search: "ERROR"                       # Using simple string to search
    recursive: true                       # Recurse other folders inside the main folder
    read_existing: false                  # Only read new files
    to: ["console", "tg:UserId-FIXME"]    # Console + Telegram UserId

  # Job 2
  - path: "/tmp/logs/ssh/"                # Path of main folder where to search
    regex: "^password=.*"                 # Using simple string to search
    recursive: true                       # Recurse other folders inside the main folder
    read_existing: true                   # Only read new files
    to: ["console"]                       # Only console 
  
  # Job 3
  - path: "/tmp/logs/nginx/access.log"    # Or path of one file
    regex: '^SUCCESS.*'                   # Using regex
    to: ["tg:UserId-FIXME"]               # Telegram UserId
```

## How to add a new notifier?

1. To add new notifier API, you just need to create new rs file like the following example: `src/notifiers/newnotifier.rs`

```rust
use anyhow::Result;
use log::{info,debug,error};
// other API import

#[derive(Clone)]
pub struct NewNotifierSink {
    bot: Bot,
    chat_id: ChatId,
}

impl std::fmt::Debug for NewNotifierSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FIXME")
    }
}

impl NewNotifierSink {
    pub fn new(token: String, chat_id: i64) -> Self {
        // FIXME create 
    }

    pub async fn send(&self, html: &str) -> Result<()> {
        info!("Sending notification from FIXME..");
        // FIXME send message
        debug!("Sent by FIXME to FIXME");
        Ok(())
    }
}
```

2. Add `pub mod newnotifier;` inside the rs file: `src/notifiers/mod.rs`
3. Add `NewNotifier(NewNotifierSink),` in the `enum Sink{}` inside the rs file: `src/notifiers/mod.rs`
4. Add `Sink::NewNotifier(s) => s.send(_text).await,` in the `send()` function inside the rs file: `src/notifiers/mod.rs`