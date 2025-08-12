use anyhow::Result;
use clap::Parser;

use env_logger::Builder;
use log::{info,debug};

use dende_rs::args::{Args, load_jobs_from_cli_or_yaml};
use dende_rs::Matcher;
use dende_rs::utils::events::spawn_job_watcher;
use dende_rs::notifiers::Notifier;

#[tokio::main]
async fn main() -> Result<()> {
    // Get arguments
    let args = Args::parse();

    // Build logger
    let level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_level(level)
        .init();
    debug!("Verbosity level: {:?}", level);

    // Build the job list (from YAML or CLI) + optional global Telegram token
    let (mut jobs, global_token) = load_jobs_from_cli_or_yaml(&args)?;

    // Start each job in a blocking thread; the notifier runs in Tokio
    let mut _thread_handles = Vec::new();

    for (idx, job) in jobs.drain(..).enumerate() {
        let token = job.telegram_token.clone().or_else(|| global_token.clone());
        let matcher = Matcher::from_spec(&job.search, &job.regex)?;

        let notifier = Notifier::new(job.to.clone(), token)?;
        let handle = spawn_job_watcher(
            idx,
            job.path.clone(),
            job.recursive,
            job.read_existing,
            matcher,
            notifier,
        );
        _thread_handles.push(handle);
    }

    info!("dende-rs: ready. Press Ctrl+C to quit..");
    let _ = tokio::signal::ctrl_c().await;
    info!("Shutdown requested. Bye!");
    Ok(())
}