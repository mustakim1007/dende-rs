<p align="center">
    <picture>
        <img src="img/dende.png" alt="dende logo" width='250' />
    </picture>
    <br>
    <strong style="font-size: large;">dende-rs (デンデ, Dende)</strong>
</p>

<hr />

Like Dende in DBZ watching over Earth, `dende-rs` watches your logs in real time and alerts the right people as soon as a line matches a string or regex. It can also run a dedicated VirusTotal watch job that polls the VT API for your payload’s hash and notifies you the instant it’s published. Configure sinks (console and/or Telegram) and run multiple jobs via CLI flags or a YAML file. The notifier layer is modular (sinks), so you can easily plug in new channels, e.g., Slack, email, SMS and without touching the core watcher. If you need more information how to add new notifier or how to use `dende-rs` please check [help page](HELP.md).

- [HELP.md](HELP.md) - How to compile it? How to install it? How to use it? How to add another API notifier?
- [CHANGELOG.md](CHANGELOG.md) - A record of all significant version changes
- [ROADMAP.md](ROADMAP.md) - List of planned evolutions
- [EXAMPLES](#examples) - Notifications examples for `log-watcher` and `virustotal-watcher`

## Quick start wth Telegram

1. Download [Rust](https://www.rust-lang.org/tools/install).
2. Create a new bot using [@Botfather](https://core.telegram.org/bots/tutorial#obtain-your-bot-token) to get a token in the format `0123456789:XXXXxXXxxxXxX3x3x-3X-XxxxX3XXXXxx3X`.
3. Put these token into YAML file or directly as CLI argument.
4. Obtain your Telegram UserId for user who would like to contact, by sending `/start` to [@userinfo](https://telegram.me/userinfobot).
5. Put these UserId YAML file like "tg:ID" or directly as CLI argument.
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
target/release/dende-rs --config ./config_example.yaml

# Single-job CLI from string search
target/release/dende-rs -- \
  -P /var/log/myapp \
  -S ERROR \
  -T console -T tg:123456789 \
  --recursive

# Single-job CLI from regex search
target/release/dende-rs -P /var/log/myapp/access.log -R "^SUCCESS.*" -T tg:123456789
```

<hr />

## Examples 

<table>
  <tr>
    <td align="center" width="50%">
      <img src="img/telegram-example-log-watcher.png" alt="Log watcher - Telegram" width="95%"><br/>
      <sub><b>Log watcher - Telegram</b></sub>
    </td>
    <td align="center" width="50%">
      <img src="img/telegram-example-virustotal-watcher.png" alt="VirusTotal watcher - Telegram" width="95%"><br/>
      <sub><b>VirusTotal watcher - Telegram</b></sub>
    </td>
  </tr>
</table>