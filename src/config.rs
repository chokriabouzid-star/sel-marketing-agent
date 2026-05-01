use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub groq_api_key:       String,
    pub reddit_client_id:   String,
    pub reddit_secret:      String,
    pub reddit_username:    String,
    pub reddit_password:    String,
    pub github_token:       String,
    pub telegram_bot_token: String,
    pub telegram_chat_id:   String,
    pub devto_api_key:      Option<String>,
    pub data_dir:           String,
    pub reddit_enabled:     bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let reddit_client_id = env::var("REDDIT_CLIENT_ID").unwrap_or_default();
        let reddit_secret    = env::var("REDDIT_SECRET").unwrap_or_default();
        let reddit_username  = env::var("REDDIT_USERNAME").unwrap_or_default();
        let reddit_password  = env::var("REDDIT_PASSWORD").unwrap_or_default();

        let reddit_enabled = !reddit_client_id.is_empty()
            && !reddit_secret.is_empty()
            && !reddit_username.is_empty()
            && !reddit_password.is_empty();

        if !reddit_enabled {
            tracing::warn!("[Config] Reddit credentials missing — Reddit disabled");
        }

        Ok(Self {
            groq_api_key: env::var("GROQ_API_KEY")
                .context("GROQ_API_KEY missing")?,
            reddit_client_id,
            reddit_secret,
            reddit_username,
            reddit_password,
            github_token: env::var("GITHUB_TOKEN")
                .context("GITHUB_TOKEN missing")?,
            telegram_bot_token: env::var("TELEGRAM_BOT_TOKEN")
                .context("TELEGRAM_BOT_TOKEN missing")?,
            telegram_chat_id: env::var("TELEGRAM_CHAT_ID")
                .context("TELEGRAM_CHAT_ID missing")?,
            devto_api_key: env::var("DEVTO_API_KEY").ok(),
            data_dir: env::var("DATA_DIR")
                .unwrap_or_else(|_| "data".to_string()),
            reddit_enabled,
        })
    }
}

pub const SEL_CONTEXT: &str = "You are Alex, a senior Rust developer. You comment on tech threads. You occasionally mention a side project called SEL Agent only when genuinely relevant. You are NOT a marketer. You do NOT promote products. You write like a real person on HN or Reddit.

SEL Agent facts (use sparingly, only when thread is directly about coding agents):
- 36/36 benchmark score across Python Go TypeScript Rust
- deterministic replay: zero LLM calls after recording
- avg 0.2 repairs per task
- built in Rust
- git snapshot rollback for safety

Your commenting style:
- Direct, no filler words
- Specific technical observations
- Short questions that show genuine curiosity
- Never promotional
- Never use: we handle, takes a different approach, significant progress,
  leverage, cutting-edge, ran into this exact issue building";

pub const KEYWORDS: &[&str] = &[
    "coding agent",
    "autonomous coding",
    "SWE-agent",
    "Devin fails",
    "LLM coding",
    "agentic coding",
    "AI code repair",
    "benchmark coding agent",
    "swe-bench",
    "code agent",
    "AI programmer",
];

pub const COMPETITOR_REPOS: &[&str] = &[
    "princeton-nlp/SWE-agent",
    "paul-gauthier/aider",
    "OpenDevin/OpenDevin",
];

pub const TARGET_SUBREDDITS: &[&str] = &[
    "LocalLLaMA",
    "programming",
    "MachineLearning",
    "rust",
    "artificial",
];
