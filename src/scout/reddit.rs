use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

use crate::config::{KEYWORDS, TARGET_SUBREDDITS};
use crate::models::{Signal, SignalSource, SignalType};

#[derive(Deserialize)]
struct RedditResp  { data: RedditData }
#[derive(Deserialize)]
struct RedditData  { children: Vec<RedditChild> }
#[derive(Deserialize)]
struct RedditChild { data: RedditPost }

#[derive(Deserialize)]
struct RedditPost {
    title:        String,
    permalink:    String,
    score:        i64,
    num_comments: i64,
    selftext:     Option<String>,
    subreddit:    String,
}

pub struct RedditScout {
    client:       Client,
    access_token: String,
}

impl RedditScout {
    pub async fn new(
        client_id: &str,
        secret:    &str,
        username:  &str,
        password:  &str,
    ) -> Result<Self> {
        let client = Client::builder()
            .user_agent("SEL-Agent-Marketing/0.1 by chokribouzid")
            .build()?;

        let tok = client
            .post("https://www.reddit.com/api/v1/access_token")
            .basic_auth(client_id, Some(secret))
            .form(&[
                ("grant_type", "password"),
                ("username",   username),
                ("password",   password),
            ])
            .send().await?
            .json::<serde_json::Value>().await?;

        let access_token = tok["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Reddit auth failed: {:?}", tok))?
            .to_string();

        info!("[Reddit] Authenticated OK");
        Ok(Self { client, access_token })
    }

    pub async fn scan(&self) -> Result<Vec<Signal>> {
        let mut signals: Vec<Signal> = Vec::new();

        for sub in TARGET_SUBREDDITS {
            let posts = self.fetch_hot(sub, 50).await?;
            for post in posts {
                let low_t = post.title.to_lowercase();
                let low_b = post.selftext.as_deref().unwrap_or("").to_lowercase();
                let ok = KEYWORDS.iter()
                    .any(|k| low_t.contains(*k) || low_b.contains(*k));
                if !ok || post.score < 5 { continue; }

                let source = match post.subreddit.to_lowercase().as_str() {
                    "localllama"      => SignalSource::RedditLocalllama,
                    "programming"     => SignalSource::RedditProgramming,
                    "machinelearning" => SignalSource::RedditMachinelearning,
                    "rust"            => SignalSource::RedditRust,
                    _                 => continue,
                };

                let mut s = Signal::new(source, SignalType::Opportunity, &post.title);
                s.url           = Some(format!("https://reddit.com{}", post.permalink));
                s.score         = Some(post.score);
                s.comment_count = Some(post.num_comments);
                if let Some(t) = &post.selftext {
                    if !t.is_empty() {
                        s.body = Some(t.chars().take(400).collect());
                    }
                }
                signals.push(s);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
        }

        info!("[Reddit] {} signals", signals.len());
        Ok(signals)
    }

    async fn fetch_hot(&self, sub: &str, limit: u32) -> Result<Vec<RedditPost>> {
        let resp = self.client
            .get(format!(
                "https://oauth.reddit.com/r/{}/hot.json?limit={}", sub, limit
            ))
            .bearer_auth(&self.access_token)
            .send().await?
            .json::<RedditResp>().await?;
        Ok(resp.data.children.into_iter().map(|c| c.data).collect())
    }
}
