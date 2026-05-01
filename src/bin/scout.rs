use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

use sel_marketing_agent::config::Config;
use sel_marketing_agent::gate::telegram::TelegramGate;
use sel_marketing_agent::ledger::Ledger;
use sel_marketing_agent::models::Signal;
use sel_marketing_agent::scout::run_all_scouts;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse()?)
        )
        .init();

    info!("=== SEL Scout ===");

    let config = Config::from_env()?;
    let ledger = Ledger::new(&config.data_dir);
    ledger.ensure_dirs().await?;

    let signals: Vec<Signal> = run_all_scouts(&config).await?;

    for s in &signals {
        ledger.append_signal(s).await?;
    }
    info!("[Scout] Saved {} signals", signals.len());

    let gate = TelegramGate::new(
        &config.telegram_bot_token,
        &config.telegram_chat_id,
    );
    gate.notify(&format!(
        "🔍 <b>Scout done</b>\n{} signals collected.",
        signals.len()
    )).await?;

    Ok(())
}
