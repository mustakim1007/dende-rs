use anyhow::Result;
use serde_json::Value;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use virustotal3::VtClient;

use std::{collections::VecDeque, sync::{Arc, Mutex}, time::Duration};
use tokio::{time::{interval, MissedTickBehavior}, sync::Semaphore};
use log::{info,debug,trace,error};

use crate::notifiers::Notifier;

#[derive(Debug, Deserialize, Clone)]
enum CheckResult {
    NotFound,
    Found {
        filename: String,
        description: String,
        url: String,
        date: String,
        reputation: i64,
        ratio: String,
        mal: u64,
    },
    TransientError,
}

/// Get score of the file
fn vt_score(v: &Value) -> (u64, u64, String) {
    let stats = &v["data"]["attributes"]["last_analysis_stats"];

    let malicious = stats["malicious"].as_u64().unwrap_or(0);
    let total: u64 = ["malicious","suspicious","undetected","harmless","timeout","failure","confirmed-timeout"]
        .iter()
        .map(|k| stats[*k].as_u64().unwrap_or(0))
        .sum();

    let ratio = if total > 0 {
        format!("{}/{}", malicious, total)
    } else {
        "0/0".to_string()
    };
    (malicious, total, ratio)
}

/// Check if the payload hash is inside VirusTotal database
async fn check_hash(vt: &VtClient<'_>, hash: &str) -> Result<CheckResult> {
    debug!("Cheking if hash '{hash}' is inside VirusTotal database..");
    match vt.get_report_file(&hash).await {
        Ok(v) => {
            // If error -> not found (file not published)
            if !v["error"].is_null() {
                info!("{hash}: not found on VirusTotal database");
                return Ok(CheckResult::NotFound);
            }

            // If no data -> not found (file not published)
            if v["data"].is_null() {
                info!("{hash}: not found on VirusTotal database");
                return Ok(CheckResult::NotFound);
            }

            // Champs utiles avec fallbacks
            let attrs = &v["data"]["attributes"];
            let filename = attrs["meaningful_name"]
                .as_str()
                .or_else(|| v["data"]["attributes"]["names"][0].as_str())
                .unwrap_or("FIXME")
                .to_string();

            // Description: signature_info.description -> popular_threat_label -> "FIXME"
            let description = attrs["signature_info"]["description"]
                .as_str()
                .or_else(|| v["data"]["popular_threat_classification"]["suggested_threat_label"].as_str())
                .unwrap_or("FIXME")
                .to_string();

            // URL 
            let url = format!("https://www.virustotal.com/gui/file/{}",hash);

            // Date: creation_date (epoch)
            let date = attrs["creation_date"]
                .as_i64()
                .or_else(|| attrs["creation_date"].as_i64()) // fallback if last_analysis_date absent
                .and_then(|ts| DateTime::<Utc>::from_timestamp(ts, 0))
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "FIXME".to_string());
            
            // Scoring
            let (mal, _total, ratio) = vt_score(&v);
            let reputation = v["data"]["attributes"]["reputation"].as_i64().unwrap_or(0);

            return Ok(CheckResult::Found { filename, description, url, date, reputation, ratio, mal })
        },
        Err(err) => {
            // Détection propre du "not found"
            let msg = err.to_string();
            if msg.contains("NotFoundError")
                || msg.contains("404")
                || msg.contains("not found")
            {
                info!("{hash}: not found on VirusTotal database");
                return Ok(CheckResult::NotFound)
            } else {
                error!("[Error] {msg}");
                return Ok(CheckResult::TransientError)
            }
        }
    }   
}

/// Scheduler for Virustotal hashes check
pub async fn spawn_virustotal_watcher(
    vt_token: &str,
    initial_hashes: Vec<String>,
    notifier: Notifier,
) -> Result<()> {

    let vt: VtClient<'_> = VtClient::new(vt_token);
    let queue = Arc::new(Mutex::new(VecDeque::from(initial_hashes)));

    // Rate limit 4/min
    let minute_sem = Arc::new(Semaphore::new(4));
    {
        let minute_sem = minute_sem.clone();
        tokio::spawn(async move {
            let mut refill = interval(Duration::from_secs(60));
            loop {
                refill.tick().await;
                let avail = minute_sem.available_permits();
                if avail < 4 {
                    minute_sem.add_permits(4 - avail);
                }
            }
        });
    }

    // Rate limit at 400/day => 1 requesst every 216s (86400/400)
    let mut tick = interval(Duration::from_secs(216));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tick.tick().await;

        let maybe_item = { queue.lock().unwrap().pop_front() };
        let Some(entry) = maybe_item else {
            info!("All hashes resolved. Done.");
            break;
        };

        // 4/min gate
        let _permit = minute_sem.acquire().await?;

        match check_hash(&vt, &entry).await? {
            CheckResult::Found { filename, description, url, date, reputation, ratio, mal } => {
                info!("Oh no! File published on VirusTotal!");
                let _txt = format!("!dende-rs::virustotal-watcher::matched!\n\nFilename: {filename}\nDescription: {description}\nURL: {url}\nDate: {date}\nCommunity reputation: {reputation}\nDetection score: {ratio} ({mal} engines flagged)");
                let _html = format!("<b>!dende-rs::virustotal-watcher::matched!</b>\n\n<b>Filename:<b> {filename}\n<b>Description:</b> {description}</b>\n<b>URL:</b> {url}\n</b>Date:</b> {date}\n<b>Community reputation:</b> {reputation}\n<b>Detection score:</b> {ratio} ({mal} engines flagged!)");
                
                trace!("\n{_txt}\n");
                notifier.notify(&_txt);
            }
            CheckResult::NotFound => {
                queue.lock().unwrap().push_back(entry);
            }
            CheckResult::TransientError => {
                error!("[Transient] {}", entry);
                queue.lock().unwrap().push_back(entry);
            }
        }
    }

    Ok(())
}