use crate::models::{Draft, DraftStatus, Signal};
use chrono::{DateTime, Utc};

// ─── Screens ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    Signals,
    Drafts,
    Published,
    Settings,
}

impl Screen {
    pub fn next(&self) -> Self {
        match self {
            Screen::Dashboard => Screen::Signals,
            Screen::Signals   => Screen::Drafts,
            Screen::Drafts    => Screen::Published,
            Screen::Published => Screen::Settings,
            Screen::Settings  => Screen::Dashboard,
        }
    }
    pub fn prev(&self) -> Self {
        match self {
            Screen::Dashboard => Screen::Settings,
            Screen::Signals   => Screen::Dashboard,
            Screen::Drafts    => Screen::Signals,
            Screen::Published => Screen::Drafts,
            Screen::Settings  => Screen::Published,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Signals   => "Signals",
            Screen::Drafts    => "Drafts",
            Screen::Published => "Published",
            Screen::Settings  => "Settings",
        }
    }
}

// ─── Edit Mode ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum EditMode {
    Normal,
    Editing,
}

// ─── Stats ────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub total_signals:     usize,
    pub processed_signals: usize,
    pub pending_drafts:    usize,
    pub approved_drafts:   usize,
    pub rejected_drafts:   usize,
    pub published_count:   usize,
    pub approval_rate:     f32,
    pub last_run:          Option<DateTime<Utc>>,
    pub next_run:          Option<DateTime<Utc>>,
}

// ─── Connection Status ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected(String),
    Disabled(String),
    NoKey,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ApiStatus {
    pub groq:     ConnectionStatus,
    pub github:   ConnectionStatus,
    pub telegram: ConnectionStatus,
    pub reddit:   ConnectionStatus,
    pub devto:    ConnectionStatus,
}

impl Default for ApiStatus {
    fn default() -> Self {
        Self {
            groq:     ConnectionStatus::Unknown,
            github:   ConnectionStatus::Unknown,
            telegram: ConnectionStatus::Unknown,
            reddit:   ConnectionStatus::Disabled("karma < 50".into()),
            devto:    ConnectionStatus::NoKey,
        }
    }
}

// ─── Confirm Action ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    RejectDraft(String),
    ClearSignals,
}

// ─── App ──────────────────────────────────────────────────────────────────

pub struct App {
    pub current_screen:   Screen,
    pub should_quit:      bool,
    pub signals:          Vec<Signal>,
    pub drafts:           Vec<Draft>,
    pub published:        Vec<Draft>,
    pub stats:            Stats,
    pub api_status:       ApiStatus,
    pub signals_cursor:   usize,
    pub drafts_cursor:    usize,
    pub published_cursor: usize,
    pub edit_mode:        EditMode,
    pub edit_buffer:      String,
    pub status_message:   Option<String>,
    pub status_is_error:  bool,
    pub show_help:        bool,
    pub show_confirm:     Option<ConfirmAction>,
    pub data_dir:         String,
}

impl App {
    pub fn new(data_dir: impl Into<String>) -> Self {
        Self {
            current_screen:   Screen::Dashboard,
            should_quit:      false,
            signals:          Vec::new(),
            drafts:           Vec::new(),
            published:        Vec::new(),
            stats:            Stats::default(),
            api_status:       ApiStatus::default(),
            signals_cursor:   0,
            drafts_cursor:    0,
            published_cursor: 0,
            edit_mode:        EditMode::Normal,
            edit_buffer:      String::new(),
            status_message:   None,
            status_is_error:  false,
            show_help:        false,
            show_confirm:     None,
            data_dir:         data_dir.into(),
        }
    }

    // ─── Navigation ───────────────────────────────────────────────────────

    pub fn next_screen(&mut self) {
        self.current_screen = self.current_screen.next();
        self.clear_status();
    }

    pub fn prev_screen(&mut self) {
        self.current_screen = self.current_screen.prev();
        self.clear_status();
    }

    // ─── Cursor ───────────────────────────────────────────────────────────

    pub fn cursor_down(&mut self) {
        match self.current_screen {
            Screen::Signals => {
                if !self.signals.is_empty() {
                    self.signals_cursor =
                        (self.signals_cursor + 1).min(self.signals.len() - 1);
                }
            }
            Screen::Published => {
                if !self.published.is_empty() {
                    self.published_cursor =
                        (self.published_cursor + 1).min(self.published.len() - 1);
                }
            }
            _ => {}
        }
    }

    pub fn cursor_up(&mut self) {
        match self.current_screen {
            Screen::Signals => {
                self.signals_cursor = self.signals_cursor.saturating_sub(1);
            }
            Screen::Published => {
                self.published_cursor = self.published_cursor.saturating_sub(1);
            }
            _ => {}
        }
    }

    // ─── Draft Navigation ─────────────────────────────────────────────────

    pub fn next_draft(&mut self) {
        let n = self.pending_drafts_count();
        if n > 0 {
            self.drafts_cursor = (self.drafts_cursor + 1) % n;
        }
    }

    pub fn prev_draft(&mut self) {
        let n = self.pending_drafts_count();
        if n > 0 {
            self.drafts_cursor =
                self.drafts_cursor.checked_sub(1).unwrap_or(n - 1);
        }
    }

    pub fn pending_drafts_count(&self) -> usize {
        self.drafts
            .iter()
            .filter(|d| d.status == DraftStatus::PendingApproval)
            .count()
    }

    pub fn current_draft(&self) -> Option<&Draft> {
        self.drafts
            .iter()
            .filter(|d| d.status == DraftStatus::PendingApproval)
            .nth(self.drafts_cursor)
    }

    pub fn current_draft_id(&self) -> Option<String> {
        self.current_draft().map(|d| d.id.clone())
    }

    // ─── Draft Actions ────────────────────────────────────────────────────

    pub fn approve_current_draft(&mut self) {
        if let Some(id) = self.current_draft_id() {
            if let Some(d) = self.drafts.iter_mut().find(|d| d.id == id) {
                d.status     = DraftStatus::Approved;
                d.ts_updated = Some(Utc::now());
            }
            let short = id[..id.len().min(16)].to_string();
            self.set_status(format!("Approved: {short}"));
            // اضبط cursor
            let n = self.pending_drafts_count();
            if n > 0 && self.drafts_cursor >= n {
                self.drafts_cursor = n - 1;
            }
            self.recalculate_stats();
        }
    }

    pub fn reject_current_draft(&mut self) {
        if let Some(id) = self.current_draft_id() {
            self.show_confirm = Some(ConfirmAction::RejectDraft(id));
        }
    }

    pub fn confirm_reject(&mut self, id: &str) {
        if let Some(d) = self.drafts.iter_mut().find(|d| d.id == id) {
            d.status     = DraftStatus::Rejected;
            d.ts_updated = Some(Utc::now());
        }
        let short = id[..id.len().min(16)].to_string();
        self.set_status(format!("Rejected: {short}"));
        self.show_confirm = None;
        let n = self.pending_drafts_count();
        if n > 0 && self.drafts_cursor >= n {
            self.drafts_cursor = n - 1;
        }
        self.recalculate_stats();
    }

    pub fn enter_edit_mode(&mut self) {
        if let Some(d) = self.current_draft() {
            self.edit_buffer = d.content.clone();
            self.edit_mode   = EditMode::Editing;
        }
    }

    pub fn save_edit(&mut self) {
        if let Some(id) = self.current_draft_id() {
            let text = self.edit_buffer.trim().to_string();
            if !text.is_empty() {
                if let Some(d) = self.drafts.iter_mut().find(|d| d.id == id) {
                    d.content    = text;
                    d.status     = DraftStatus::Approved;
                    d.ts_updated = Some(Utc::now());
                }
                self.set_status("Edited and approved".into());
            }
        }
        self.edit_mode   = EditMode::Normal;
        self.edit_buffer = String::new();
        self.recalculate_stats();
    }

    pub fn cancel_edit(&mut self) {
        self.edit_mode   = EditMode::Normal;
        self.edit_buffer = String::new();
    }

    // ─── Status Bar ───────────────────────────────────────────────────────

    pub fn set_status(&mut self, msg: String) {
        self.status_message  = Some(msg);
        self.status_is_error = false;
    }

    pub fn set_error(&mut self, msg: String) {
        self.status_message  = Some(msg);
        self.status_is_error = true;
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    // ─── Stats ────────────────────────────────────────────────────────────

    pub fn recalculate_stats(&mut self) {
        let approved = self.drafts.iter()
            .filter(|d| d.status == DraftStatus::Approved).count();
        let rejected = self.drafts.iter()
            .filter(|d| d.status == DraftStatus::Rejected).count();
        let pending  = self.drafts.iter()
            .filter(|d| d.status == DraftStatus::PendingApproval).count();
        let total    = self.drafts.len();

        self.stats.pending_drafts    = pending;
        self.stats.approved_drafts   = approved;
        self.stats.rejected_drafts   = rejected;
        self.stats.total_signals     = self.signals.len();
        self.stats.processed_signals = self.signals.iter()
            .filter(|s| s.processed).count();
        self.stats.published_count   = self.published.len();
        self.stats.approval_rate     = if total > 0 {
            approved as f32 / total as f32
        } else {
            0.0
        };
    }

    // ─── Data I/O — نستخدم tokio::runtime لأن TUI sync ─────────────────

    pub fn load_data(&mut self) -> anyhow::Result<()> {
        use crate::ledger::Ledger;

        let ledger = Ledger::new(&self.data_dir);
        let rt     = tokio::runtime::Runtime::new()?;

        let signals    = rt.block_on(ledger.load_signals())?;
        let all_drafts = rt.block_on(ledger.load_drafts())?;

        self.signals = signals;
        self.drafts  = all_drafts
            .iter()
            .filter(|d| d.status != DraftStatus::Published)
            .cloned()
            .collect();
        self.published = all_drafts
            .into_iter()
            .filter(|d| d.status == DraftStatus::Published)
            .collect();

        self.recalculate_stats();
        Ok(())
    }

    pub fn save_data(&mut self) -> anyhow::Result<()> {
        use crate::ledger::Ledger;

        let ledger     = Ledger::new(&self.data_dir);
        let rt         = tokio::runtime::Runtime::new()?;
        let mut all    = self.drafts.clone();
        all.extend(self.published.clone());

        // أعد كتابة drafts.jsonl بالكامل
        rt.block_on(async {
            ledger.ensure_dirs().await?;
            // احذف القديم وأعد الكتابة عبر update لكل draft
            // الطريقة الأبسط: استخدم rewrite عبر update_draft_status
            // لكن ليس لدينا rewrite_all مباشرة
            // نستخدم بديل: احفظ كل draft بـ update_draft_status
            for d in &all {
                ledger.update_draft_status(&d.id, d.status.clone()).await.ok();
                if d.status == DraftStatus::Approved {
                    ledger.update_draft_content(&d.id, &d.content).await.ok();
                }
            }
            anyhow::Ok(())
        })?;

        self.set_status("Saved".into());
        Ok(())
    }
}
