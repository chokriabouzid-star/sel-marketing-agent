use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

use sel_marketing_agent::config::Config;
use sel_marketing_agent::dispatch::Dispatcher;
use sel_marketing_agent::ledger::Ledger;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse()?)
        )
        .init();

    info!("=== SEL Dispatch ===");

    let config     = Config::from_env()?;
    let ledger     = Ledger::new(&config.data_dir);
    let dispatcher = Dispatcher::new(config, ledger);
    let count      = dispatcher.run().await?;

    info!("[Dispatch] {} posts published", count);
    Ok(())
}
