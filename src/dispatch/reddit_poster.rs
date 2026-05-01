use anyhow::Result;
use reqwest::Client;
use tracing::info;

pub struct RedditPoster {
    client: Client,
    token:  String,
}

impl RedditPoster {
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

        let token = tok["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Reddit auth failed"))?
            .to_string();

        Ok(Self { client, token })
    }

    pub async fn post_comment(
        &self,
        thing_id: &str,
        text:     &str,
    ) -> Result<String> {
        let resp = self.client
            .post("https://oauth.reddit.com/api/comment")
            .bearer_auth(&self.token)
            .form(&[
                ("api_type", "json"),
                ("thing_id", thing_id),
                ("text",     text),
            ])
            .send().await?
            .json::<serde_json::Value>().await?;

        let url = resp["json"]["data"]["things"][0]["data"]["permalink"]
            .as_str()
            .map(|p| format!("https://reddit.com{}", p))
            .unwrap_or_else(|| "https://reddit.com".to_string());

        info!("[Reddit Poster] {}", url);
        Ok(url)
    }
}
