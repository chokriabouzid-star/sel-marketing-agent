use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

use crate::config::COMPETITOR_REPOS;
use crate::models::{Signal, SignalSource, SignalType};

#[derive(Deserialize)]
struct GhRelease {
    name:         Option<String>,
    tag_name:     String,
    body:         Option<String>,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
    html_url:     String,
}

pub struct GithubScanner {
    client: Client,
    token:  String,
}

impl GithubScanner {
    pub fn new(token: impl Into<String>) -> Self {
        Self { client: Client::new(), token: token.into() }
    }

    pub async fn scan_competitors(&self) -> Result<Vec<Signal>> {
        let mut signals: Vec<Signal> = Vec::new();

        for repo in COMPETITOR_REPOS {
            match self.latest_release(repo).await {
                Ok(Some(r)) => {
                    let title = format!(
                        "[{}] {}",
                        repo,
                        r.name.as_deref().unwrap_or(&r.tag_name)
                    );
                    let mut s = Signal::new(
                        SignalSource::GithubCompetitor(repo.to_string()),
                        SignalType::CompetitorNews,
                        title,
                    );
                    s.url  = Some(r.html_url);
                    s.body = r.body.map(|b| b.chars().take(500).collect());
                    if let Some(ts) = r.published_at { s.ts = ts; }
                    signals.push(s);
                }
                Ok(None) => {}
                Err(e)   => tracing::warn!("[GitHub] {}: {}", repo, e),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        info!("[GitHub] {} signals", signals.len());
        Ok(signals)
    }

    async fn latest_release(&self, repo: &str) -> Result<Option<GhRelease>> {
        let resp = self.client
            .get(format!(
                "https://api.github.com/repos/{}/releases/latest", repo
            ))
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent",     "SEL-Marketing-Agent/0.1")
            .header("Accept",         "application/vnd.github.v3+json")
            .send().await?;

        if resp.status() == 404 { return Ok(None); }
        Ok(Some(resp.json::<GhRelease>().await?))
    }
}
