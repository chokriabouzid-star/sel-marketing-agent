use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

use sel_marketing_agent::config::Config;
use sel_marketing_agent::gate::process_updates;
use sel_marketing_agent::ledger::Ledger;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse()?)
        )
        .init();

    info!("=== SEL Gate ===");

    let config   = Config::from_env()?;
    let ledger   = Ledger::new(&config.data_dir);
    let mut last_id: i64 = 0;

    let processed = process_updates(
        &config.telegram_bot_token,
        &config.telegram_chat_id,
        &ledger,
        &mut last_id,
    ).await?;

    info!("[Gate] {} commands processed", processed);
    Ok(())
}
