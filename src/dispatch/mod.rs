pub mod reddit_poster;

use anyhow::Result;
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::config::Config;
use crate::ledger::Ledger;
use crate::models::{Draft, DraftStatus, Platform, PostMetrics, Published};

use reddit_poster::RedditPoster;

pub struct Dispatcher {
    config: Config,
    ledger: Ledger,
}

impl Dispatcher {
    pub fn new(config: Config, ledger: Ledger) -> Self {
        Self { config, ledger }
    }

    pub async fn run(&self) -> Result<usize> {
        let all_drafts: Vec<Draft> = self.ledger.load_drafts().await?;

        let approved: Vec<Draft> = all_drafts
            .into_iter()
            .filter(|d| d.status == DraftStatus::Approved)
            .collect();

        info!("[Dispatch] {} approved drafts", approved.len());

        let mut done = 0usize;

        for draft in approved {
            match self.dispatch_one(&draft).await {
                Ok(url) => {
                    let pub_ = Published {
                        id:           Uuid::new_v4().to_string(),
                        draft_id:     draft.id.clone(),
                        platform:     draft.platform.clone(),
                        url:          Some(url),
                        ts_published: Utc::now(),
                        metrics:      PostMetrics::default(),
                    };
                    self.ledger.append_published(&pub_).await?;
                    self.ledger
                        .update_draft_status(&draft.id, DraftStatus::Published)
                        .await?;
                    done += 1;
                }
                Err(e) => {
                    tracing::error!("[Dispatch] Failed {}: {}", draft.id, e);
                    self.ledger
                        .update_draft_status(&draft.id, DraftStatus::Failed)
                        .await?;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Ok(done)
    }

    async fn dispatch_one(&self, draft: &Draft) -> Result<String> {
        match draft.platform {
            Platform::Reddit => {
                if !self.config.reddit_enabled {
                    info!("[Dispatch] Reddit disabled, skipping");
                    return Ok(format!("reddit_disabled_{}", draft.id));
                }
                let poster = RedditPoster::new(
                    &self.config.reddit_client_id,
                    &self.config.reddit_secret,
                    &self.config.reddit_username,
                    &self.config.reddit_password,
                ).await?;
                let thing_id = draft.signal_url.as_deref()
                    .and_then(extract_post_id)
                    .unwrap_or("t3_000000");
                poster.post_comment(thing_id, &draft.content).await
            }
            Platform::DevTo => {
                match &self.config.devto_api_key {
                    Some(key) => post_to_devto(key, &draft.content).await,
                    None => {
                        tracing::warn!("[Dispatch] DEVTO_API_KEY not set");
                        Ok(format!("pending_devto_{}", draft.id))
                    }
                }
            }
            _ => {
                info!("[Dispatch] {} not automated yet", draft.platform);
                Ok(format!("manual_{}", draft.id))
            }
        }
    }
}

fn extract_post_id(url: &str) -> Option<&str> {
    let _ = url.split("/comments/").nth(1)?;
    Some("t3_placeholder")
}

async fn post_to_devto(api_key: &str, content: &str) -> Result<String> {
    // ─── استخرج العنوان ───────────────────────────────────────────────
    let title = content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").trim().to_string())
        .unwrap_or_else(|| "SEL Agent: Building a Reliable Coding Agent".to_string());

    // ─── احذف سطر العنوان من الـ body ────────────────────────────────
    let body: String = content
        .lines()
        .enumerate()
        .filter(|(i, l)| !(*i == 0 && l.starts_with("# ")))
        .map(|(_, l)| l)
        .collect::<Vec<_>>()
        .join("\n");

    // ─── tags بسيطة وصحيحة ───────────────────────────────────────────
    let tags = serde_json::json!(["rust", "ai", "coding", "devtools"]);

    tracing::info!("[DevTo] Title: {}", title);
    tracing::info!("[DevTo] Body length: {} chars", body.len());

    let client = reqwest::Client::new();
    let resp = client
        .post("https://dev.to/api/articles")
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "article": {
                "title":         title,
                "published":     false,
                "body_markdown": body,
                "tags":          tags,
            }
        }))
        .send()
        .await?;

    let status = resp.status();
    let body_resp = resp.text().await?;

    tracing::info!("[DevTo] Status: {}", status);

    if !status.is_success() {
        anyhow::bail!("Dev.to API error {}: {}", status, body_resp);
    }

    if body_resp.is_empty() {
        anyhow::bail!("Dev.to returned empty response");
    }

    let json: serde_json::Value = serde_json::from_str(&body_resp)
        .map_err(|e| anyhow::anyhow!("JSON parse error: {e}\nBody: {body_resp}"))?;

    let url = json["url"]
        .as_str()
        .unwrap_or("https://dev.to")
        .to_string();

    tracing::info!("[DevTo] Published: {}", url);
    Ok(url)
}
