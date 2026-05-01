use anyhow::Result;
use std::path::Path;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::info;

use crate::models::{Draft, DraftStatus, Published, Signal};

pub struct Ledger {
    data_dir: String,
}

impl Ledger {
    pub fn new(data_dir: impl Into<String>) -> Self {
        Self { data_dir: data_dir.into() }
    }

    pub async fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir).await?;
        Ok(())
    }

    fn path(&self, filename: &str) -> String {
        format!("{}/{}", self.data_dir, filename)
    }

    pub async fn append_signal(&self, signal: &Signal) -> Result<()> {
        self.append_line("signals.jsonl", &serde_json::to_string(signal)?).await
    }

    pub async fn load_signals(&self) -> Result<Vec<Signal>> {
        self.load_jsonl("signals.jsonl").await
    }

    pub async fn load_unprocessed_signals(&self) -> Result<Vec<Signal>> {
        let all: Vec<Signal> = self.load_signals().await?;
        Ok(all.into_iter().filter(|s| !s.processed).collect())
    }

    pub async fn append_draft(&self, draft: &Draft) -> Result<()> {
        self.append_line("drafts.jsonl", &serde_json::to_string(draft)?).await
    }

    pub async fn load_drafts(&self) -> Result<Vec<Draft>> {
        self.load_jsonl("drafts.jsonl").await
    }

    pub async fn load_pending_drafts(&self) -> Result<Vec<Draft>> {
        let all: Vec<Draft> = self.load_drafts().await?;
        Ok(all.into_iter()
            .filter(|d| d.status == DraftStatus::PendingApproval)
            .collect())
    }

    pub async fn update_draft_status(
        &self,
        draft_id: &str,
        status: DraftStatus,
    ) -> Result<bool> {
        let mut drafts: Vec<Draft> = self.load_drafts().await?;
        let mut found = false;
        for d in &mut drafts {
            if d.id == draft_id {
                d.status     = status.clone();
                d.ts_updated = Some(chrono::Utc::now());
                found        = true;
                break;
            }
        }
        if found {
            self.rewrite_jsonl("drafts.jsonl", &drafts).await?;
            info!("Draft {} -> {:?}", draft_id, status);
        }
        Ok(found)
    }

    pub async fn update_draft_content(
        &self,
        draft_id: &str,
        new_content: &str,
    ) -> Result<bool> {
        let mut drafts: Vec<Draft> = self.load_drafts().await?;
        let mut found = false;
        for d in &mut drafts {
            if d.id == draft_id {
                d.content    = new_content.to_string();
                d.ts_updated = Some(chrono::Utc::now());
                found        = true;
                break;
            }
        }
        if found {
            self.rewrite_jsonl("drafts.jsonl", &drafts).await?;
        }
        Ok(found)
    }

    pub async fn append_published(&self, p: &Published) -> Result<()> {
        self.append_line("published.jsonl", &serde_json::to_string(p)?).await
    }

    pub async fn load_published(&self) -> Result<Vec<Published>> {
        self.load_jsonl("published.jsonl").await
    }

    async fn append_line(&self, filename: &str, line: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path(filename))
            .await?;
        file.write_all(format!("{}\n", line).as_bytes()).await?;
        Ok(())
    }

    async fn load_jsonl<T>(&self, filename: &str) -> Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let path = self.path(filename);
        if !Path::new(&path).exists() {
            return Ok(vec![]);
        }
        let file = fs::File::open(&path).await?;
        let mut lines = BufReader::new(file).lines();
        let mut items: Vec<T> = Vec::new();
        while let Some(line) = lines.next_line().await? {
            let t = line.trim().to_string();
            if t.is_empty() {
                continue;
            }
            match serde_json::from_str::<T>(&t) {
                Ok(item) => items.push(item),
                Err(e)   => tracing::warn!("Bad line in {}: {}", filename, e),
            }
        }
        Ok(items)
    }

    async fn rewrite_jsonl<T: serde::Serialize>(
        &self,
        filename: &str,
        items: &[T],
    ) -> Result<()> {
        let mut out = String::new();
        for item in items {
            out.push_str(&serde_json::to_string(item)?);
            out.push('\n');
        }
        fs::write(self.path(filename), out).await?;
        Ok(())
    }
}
