use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

use sel_marketing_agent::config::Config;
use sel_marketing_agent::forge::Forge;
use sel_marketing_agent::gate::telegram::TelegramGate;
use sel_marketing_agent::ledger::Ledger;
use sel_marketing_agent::models::{Draft, Signal};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse()?)
        )
        .init();

    info!("=== SEL Forge ===");

    let config = Config::from_env()?;
    let ledger = Ledger::new(&config.data_dir);
    let gate   = TelegramGate::new(
        &config.telegram_bot_token,
        &config.telegram_chat_id,
    );

    let unprocessed: Vec<Signal> = ledger.load_unprocessed_signals().await?;
    info!("[Forge] {} unprocessed signals", unprocessed.len());

    if unprocessed.is_empty() {
        info!("[Forge] Nothing to forge. Exiting.");
        return Ok(());
    }

    let forge          = Forge::new(&config.groq_api_key);
    let drafts: Vec<Draft> = forge.generate_batch(&unprocessed, 5).await;

    for draft in &drafts {
        ledger.append_draft(draft).await?;
        gate.send_draft(draft).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    info!("[Forge] {} drafts sent for approval", drafts.len());
    Ok(())
}
