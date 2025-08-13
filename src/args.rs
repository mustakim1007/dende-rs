use anyhow::{Result, Context};
use clap::{ArgAction, Parser};
use serde::Deserialize;
use std::path::PathBuf;

/// CLI arguments for single-job mode or --config YAML multi-job mode.
#[derive(Parser, Debug)]
#[command(name = "dende-rs (デンデ, Dende)")]
#[command(author = "@g0h4n_0")]
#[command(about = "Monitor logs files and folders and notify from Telegram bot when a search or regex matches!")]
pub struct Args {
    /// Path to watch (file or folder, single-job CLI mode)
    #[arg(short = 'P', long = "path")]
    pub path: Option<PathBuf>,

    /// Literal term to search for (single-job CLI mode)
    #[arg(short = 'S', long = "search")]
    pub search: Option<String>,

    /// Regular expression (Rust regex, single-job CLI mode)
    #[arg(short = 'R', long = "regex")]
    pub regex: Option<String>,

    /// Recipients: plain text = console tag, 'tg:<CHAT_ID>' = Telegram (single-job CLI mode)
    #[arg(short = 'T', long = "to", value_delimiter = ',')]
    pub to: Vec<String>,

    /// Watch subdirectories (single-job CLI mode)
    #[arg(long = "recursive", default_value_t = false)]
    pub recursive: bool,

    /// On startup, read existing files from the beginning (single-job CLI mode)
    #[arg(long = "read-existing", default_value_t = true)]
    pub read_existing: bool,

    /// Telegram bot token (or ENV TELEGRAM_BOT_TOKEN) (single-job CLI mode)
    #[arg(long = "telegram-token", env = "TELEGRAM_BOT_TOKEN")]
    pub telegram_token: Option<String>,

    /// YAML configuration file (multi-jobs)
    #[arg(short = 'C', long = "config")]
    pub config: Option<PathBuf>,

    /// Verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', action = ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JobSpec {
    pub path: PathBuf,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub regex: Option<String>,
    pub to: Vec<String>,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default = "default_true")]
    pub read_existing: bool,
    #[serde(default)]
    pub telegram_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub telegram_token: Option<String>,
    pub jobs: Vec<JobSpec>,
}

fn default_true() -> bool { true }

/// Load jobs from YAML if provided, otherwise build a single job from CLI args.
pub fn load_jobs_from_cli_or_yaml(args: &Args) -> Result<(Vec<JobSpec>, Option<String>)> {
    if let Some(cfg_path) = args.config.as_ref() {
        let text = std::fs::read_to_string(cfg_path)
            .with_context(|| format!("Reading config file: {}", cfg_path.display()))?;
        let cfg: ConfigFile = serde_yaml::from_str(&text)
            .with_context(|| "Parsing YAML configuration")?;

        if cfg.jobs.is_empty() {
            anyhow::bail!("YAML file contains no jobs.");
        }
        for (i, j) in cfg.jobs.iter().enumerate() {
            if !j.path.is_dir() && !j.path.is_file() {
                anyhow::bail!("Job #{i}: non-existent file or directory on path: {}", j.path.display());
            }
            if j.search.is_none() && j.regex.is_none() {
                anyhow::bail!("Job #{i}: specify 'search' or 'regex'.");
            }
            if j.to.is_empty() {
                anyhow::bail!("Job #{i}: specify at least one recipient ('to').");
            }
        }
        Ok((cfg.jobs, cfg.telegram_token))
    } else {
        // Single-job CLI mode
        let path = args.path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("--path required in CLI mode (or use --config)"))?;
        if !path.is_dir() {
            anyhow::bail!("--path must be an existing directory");
        }
        if args.search.is_none() && args.regex.is_none() {
            anyhow::bail!("Specify --search or --regex (or use --config).");
        }
        if args.to.is_empty() {
            anyhow::bail!("Specify at least one recipient via -D/--dest.");
        }

        let job = JobSpec {
            path: path.clone(),
            search: args.search.clone(),
            regex: args.regex.clone(),
            to: args.to.clone(),
            recursive: args.recursive,
            read_existing: args.read_existing,
            telegram_token: args.telegram_token.clone(),
        };
        Ok((vec![job], None))
    }
}