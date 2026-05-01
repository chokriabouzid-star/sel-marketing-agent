use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const GROQ_URL: &str = "https://api.groq.com/openai/v1/chat/completions";
const MODEL_A:  &str = "llama-3.3-70b-versatile";
const MODEL_B:  &str = "llama-3.1-8b-instant";
const MODEL_C:  &str = "gemma2-9b-it";

#[derive(Serialize)]
struct GroqReq {
    model:       String,
    messages:    Vec<GroqMsg>,
    max_tokens:  u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct GroqMsg {
    role:    String,
    content: String,
}

#[derive(Deserialize)]
struct GroqResp {
    choices: Vec<GroqChoice>,
}

#[derive(Deserialize)]
struct GroqChoice {
    message: GroqMsgContent,
}

#[derive(Deserialize)]
struct GroqMsgContent {
    content: String,
}

pub struct GroqClient {
    client:  Client,
    api_key: String,
}

impl GroqClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { client: Client::new(), api_key: api_key.into() }
    }

    pub async fn complete(
        &self,
        system:      &str,
        user:        &str,
        max_tokens:  u32,
        temperature: f32,
    ) -> Result<String> {
        for model in [MODEL_A, MODEL_B, MODEL_C] {
            match self.call(model, system, user, max_tokens, temperature).await {
                Ok(r) => {
                    tracing::info!("[Groq] Used model: {}", model);
                    return Ok(r);
                }
                Err(e) => {
                    tracing::warn!("[Groq] Model {} failed: {}", model, e);
                    continue;
                }
            }
        }
        anyhow::bail!("[Groq] All models failed")
    }

    async fn call(
        &self,
        model:       &str,
        system:      &str,
        user:        &str,
        max_tokens:  u32,
        temperature: f32,
    ) -> Result<String> {
        let resp = self.client
            .post(GROQ_URL)
            .bearer_auth(&self.api_key)
            .json(&GroqReq {
                model:    model.to_string(),
                messages: vec![
                    GroqMsg { role: "system".into(), content: system.into() },
                    GroqMsg { role: "user".into(),   content: user.into()   },
                ],
                max_tokens,
                temperature,
            })
            .send().await?;

        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("Groq {}: {}", status, resp.text().await?);
        }

        resp.json::<GroqResp>().await?
            .choices.into_iter().next()
            .map(|c| c.message.content)
            .context("Empty Groq response")
    }
}
