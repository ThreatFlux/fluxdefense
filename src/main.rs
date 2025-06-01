use fluxdefense::FluxDefense;
use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("FluxDefense EDR starting...");
    
    let mut defense = match FluxDefense::new() {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to initialize FluxDefense: {}", e);
            return Err(e);
        }
    };
    
    if let Err(e) = defense.start().await {
        error!("Failed to start protection: {}", e);
        return Err(e);
    }
    
    // Keep running until interrupted
    tokio::signal::ctrl_c().await?;
    
    info!("Received shutdown signal");
    defense.stop().await?;
    
    Ok(())
}