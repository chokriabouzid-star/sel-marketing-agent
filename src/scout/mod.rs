pub mod github_scanner;
pub mod hackernews;
pub mod reddit;

use anyhow::Result;
use std::collections::HashSet;
use tracing::info;

use crate::config::Config;
use crate::models::Signal;

use github_scanner::GithubScanner;
use hackernews::HackerNewsScout;
use reddit::RedditScout;

pub async fn run_all_scouts(config: &Config) -> Result<Vec<Signal>> {
    info!("[Scout] Starting...");

    let hn_scout   = HackerNewsScout::new();
    let gh_scanner = GithubScanner::new(&config.github_token);

    let mut all: Vec<Signal> = Vec::new();

    // Reddit — فقط إذا كانت credentials موجودة
    if config.reddit_enabled {
        info!("[Scout] Reddit enabled — connecting...");
        match RedditScout::new(
            &config.reddit_client_id,
            &config.reddit_secret,
            &config.reddit_username,
            &config.reddit_password,
        ).await {
            Ok(scout) => {
                match scout.scan().await {
                    Ok(s)  => { info!("[Reddit] {} signals", s.len()); all.extend(s); }
                    Err(e) => tracing::warn!("[Scout] Reddit scan failed: {}", e),
                }
            }
            Err(e) => tracing::warn!("[Scout] Reddit auth failed: {}", e),
        }
    } else {
        info!("[Scout] Reddit disabled — skipping");
    }

    // HN و GitHub بالتوازي
    let (h_result, g_result) = tokio::join!(
        hn_scout.scan(),
        gh_scanner.scan_competitors(),
    );

    // GitHub أولاً — لا dedup عليها (URLs فريدة دائماً)
    match g_result {
        Ok(s) => {
            info!("[GitHub] {} competitor signals", s.len());
            all.extend(s);
        }
        Err(e) => tracing::warn!("[Scout] GitHub failed: {}", e),
    }

    // HN مع dedup
    match h_result {
        Ok(s) => {
            info!("[HN] {} signals before dedup", s.len());
            all.extend(s);
        }
        Err(e) => tracing::warn!("[Scout] HN failed: {}", e),
    }

    // dedup بناءً على URL
    let mut seen: HashSet<String> = HashSet::new();
    let before = all.len();
    all.retain(|s| {
        match &s.url {
            Some(url) => seen.insert(url.clone()),
            None      => true,
        }
    });
    let after = before - all.len();
    if after > 0 {
        info!("[Scout] Removed {} duplicates", after);
    }

    info!("[Scout] {} unique signals total", all.len());
    Ok(all)
}
