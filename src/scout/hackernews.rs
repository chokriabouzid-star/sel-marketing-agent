use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

use crate::config::KEYWORDS;
use crate::models::{Signal, SignalSource, SignalType};

#[derive(Deserialize)]
struct HnResponse {
    hits: Vec<HnHit>,
}

#[derive(Deserialize)]
struct HnHit {
    #[serde(rename = "objectID")]
    object_id:    String,
    title:        Option<String>,
    points:       Option<i64>,
    num_comments: Option<i64>,
    story_text:   Option<String>,
}

pub struct HackerNewsScout {
    client: Client,
}

impl HackerNewsScout {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub async fn scan(&self) -> Result<Vec<Signal>> {
        let mut signals: Vec<Signal> = Vec::new();

        for kw in KEYWORDS {
            let resp = self.client
                .get("https://hn.algolia.com/api/v1/search_by_date")
                .query(&[
                    ("query",        *kw),
                    ("tags",         "story"),
                    ("hitsPerPage",  "15"),
                ])
                .send().await?
                .json::<HnResponse>().await?;

            for hit in resp.hits {
                if hit.points.unwrap_or(0) < 10 { continue; }
                let title = match hit.title {
                    Some(t) if !t.is_empty() => t,
                    _ => continue,
                };
                let mut s = Signal::new(
                    SignalSource::HackerNews,
                    SignalType::Opportunity,
                    &title,
                );
                s.url = Some(format!(
                    "https://news.ycombinator.com/item?id={}", hit.object_id
                ));
                s.score         = hit.points;
                s.comment_count = hit.num_comments;
                if let Some(text) = hit.story_text {
                    s.body = Some(text.chars().take(300).collect());
                }
                signals.push(s);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        }

        info!("[HN] {} signals", signals.len());
        Ok(signals)
    }
}
