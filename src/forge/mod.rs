pub mod groq_client;
pub mod templates;

use anyhow::Result;
use tracing::info;

use crate::config::SEL_CONTEXT;
use crate::models::{ContentType, Draft, Signal};

use groq_client::GroqClient;
use templates::{build_prompt, select_platform};

pub struct Forge {
    groq: GroqClient,
}

impl Forge {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { groq: GroqClient::new(api_key) }
    }

    pub async fn generate_draft(&self, signal: &Signal) -> Result<Draft> {
        let platform    = select_platform(signal);
        let user_prompt = build_prompt(signal, &platform);

        let raw: String = self.groq.complete(
            SEL_CONTEXT,
            &user_prompt,
            450,
            0.72,
        ).await?;

        if raw.trim() == "SKIP" {
            anyhow::bail!("Model skipped: {}", signal.title);
        }

        let mut draft = Draft::new(platform, ContentType::Comment, raw.trim());
        draft.signal_id     = Some(signal.id.clone());
        draft.signal_url    = signal.url.clone();
        draft.quality_score = Some(self.quality_score(&draft.content));

        info!(
            "[Forge] Draft ready q={:.2} — {}",
            draft.quality_score.unwrap_or(0.0),
            &signal.title[..signal.title.len().min(55)]
        );
        Ok(draft)
    }

pub async fn generate_batch(
    &self,
    signals: &[Signal],
    max:     usize,
) -> Vec<Draft> {
    let mut drafts: Vec<Draft> = Vec::new();
    let mut processed_urls: std::collections::HashSet<String> = 
        std::collections::HashSet::new();

    for signal in signals.iter() {
        if drafts.len() >= max { break; }

        // تجاهل نفس الـ URL إذا عولجت مسبقاً في هذا الـ run
        let url_key = signal.url.clone()
            .unwrap_or_else(|| signal.id.clone());
        if processed_urls.contains(&url_key) {
            tracing::info!("[Forge] Dedup skip: {}", &signal.title[..signal.title.len().min(50)]);
            continue;
        }
        processed_urls.insert(url_key);

        match self.generate_draft(signal).await {
            Ok(d)  => drafts.push(d),
            Err(e) => tracing::warn!("[Forge] Skip: {}", e),
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    drafts
}
    fn quality_score(&self, content: &str) -> f32 {
        let mut s: f32 = 0.5;
        if content.chars().any(|c| c.is_ascii_digit()) { s += 0.15; }
        if !content.contains('\u{2014}')               { s += 0.10; }
        let wc = content.split_whitespace().count();
        if (25..=130).contains(&wc)                     { s += 0.15; }
        let bad = ["leverage", "synergy", "cutting-edge", "game-changing"];
        if !bad.iter().any(|w| content.to_lowercase().contains(w)) { s += 0.10; }
        s.min(1.0)
    }
}
