use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

use sel_marketing_agent::config::Config;
use sel_marketing_agent::gate::telegram::TelegramGate;
use sel_marketing_agent::ledger::Ledger;
use sel_marketing_agent::models::{Draft, DraftStatus, Published, Signal};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse()?)
        )
        .init();

    info!("=== SEL Weekly Report ===");

    let config = Config::from_env()?;
    let ledger = Ledger::new(&config.data_dir);
    let gate   = TelegramGate::new(
        &config.telegram_bot_token,
        &config.telegram_chat_id,
    );

    let signals:   Vec<Signal>    = ledger.load_signals().await?;
    let drafts:    Vec<Draft>     = ledger.load_drafts().await?;
    let published: Vec<Published> = ledger.load_published().await?;

    let approved = drafts.iter().filter(|d| {
        d.status == DraftStatus::Approved || d.status == DraftStatus::Published
    }).count();

    gate.weekly_report(
        signals.len(),
        drafts.len(),
        approved,
        published.len(),
    ).await?;

    info!("[Report] Sent successfully");
    Ok(())
}
