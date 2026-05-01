use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use tracing::info;

use crate::models::{ContentType, Draft, Platform};

pub struct TelegramGate {
    client:    Client,
    bot_token: String,
    chat_id:   String,
}

impl TelegramGate {
    pub fn new(bot_token: impl Into<String>, chat_id: impl Into<String>) -> Self {
        Self {
            client:    Client::new(),
            bot_token: bot_token.into(),
            chat_id:   chat_id.into(),
        }
    }

    fn url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.bot_token, method)
    }

    pub async fn send_draft(&self, d: &Draft) -> Result<()> {
        let emoji = match d.platform {
            Platform::Reddit     => "💬",
            Platform::HackerNews => "🟠",
            Platform::DevTo      => "📝",
            Platform::GitHub     => "🐙",
            Platform::Twitter    => "🐦",
        };
        let kind = match d.content_type {
            ContentType::Comment       => "Comment",
            ContentType::Post          => "Post",
            ContentType::Article       => "Article",
            ContentType::ReadmeSection => "README",
        };
        let quality = match d.quality_score.unwrap_or(0.0) {
            s if s >= 0.85 => "🟢 Excellent",
            s if s >= 0.70 => "🟡 Good",
            s if s >= 0.55 => "🟠 Fair",
            _              => "🔴 Review needed",
        };
        let link = d.signal_url.as_deref()
            .map(|u| format!("\n🔗 <a href=\"{}\">Signal</a>", u))
            .unwrap_or_default();

        let msg = format!(
            "{emoji} <b>SEL Marketing Agent</b>\n\
            Platform: <code>{platform}</code> | Type: <code>{kind}</code>\n\
            Quality: {quality}{link}\n\n\
            <pre>{content}</pre>\n\n\
            <i>ID: {id}</i>\n\n\
            approve {id}\n\
            edit {id} [new text]\n\
            reject {id}",
            emoji    = emoji,
            platform = d.platform,
            kind     = kind,
            quality  = quality,
            link     = link,
            content  = html_escape(&d.content),
            id       = d.id,
        );

        self.client
            .post(self.url("sendMessage"))
            .json(&json!({
                "chat_id":                  self.chat_id,
                "text":                     msg,
                "parse_mode":               "HTML",
                "disable_web_page_preview": true,
            }))
            .send().await?
            .error_for_status()?;

        info!("[Gate] Sent draft {}", d.id);
        Ok(())
    }

    pub async fn notify(&self, html: &str) -> Result<()> {
        self.client
            .post(self.url("sendMessage"))
            .json(&json!({
                "chat_id":    self.chat_id,
                "text":       html,
                "parse_mode": "HTML",
            }))
            .send().await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn weekly_report(
        &self,
        signals:   usize,
        generated: usize,
        approved:  usize,
        published: usize,
    ) -> Result<()> {
        let rate = if generated > 0 {
            (approved as f32 / generated as f32) * 100.0
        } else {
            0.0
        };
        self.notify(&format!(
            "📊 <b>Weekly Report</b>\n\n\
            Signals:   <b>{signals}</b>\n\
            Generated: <b>{generated}</b>\n\
            Approved:  <b>{approved}</b>\n\
            Published: <b>{published}</b>\n\
            Rate:      <b>{rate:.1}%</b>",
        )).await
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
}

#[derive(Debug)]
pub enum GateCommand {
    Approve(String),
    Reject(String),
    Edit { id: String, new_content: String },
    Unknown,
}

pub fn parse_command(text: &str) -> GateCommand {
    let t = text.trim();
    if let Some(rest) = t.strip_prefix("approve ") {
        return GateCommand::Approve(rest.trim().to_string());
    }
    if let Some(rest) = t.strip_prefix("reject ") {
        return GateCommand::Reject(rest.trim().to_string());
    }
    if let Some(rest) = t.strip_prefix("edit ") {
        let mut parts = rest.splitn(2, ' ');
        if let (Some(id), Some(content)) = (parts.next(), parts.next()) {
            return GateCommand::Edit {
                id:          id.to_string(),
                new_content: content.to_string(),
            };
        }
    }
    GateCommand::Unknown
}
