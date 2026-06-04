mod api;
mod server;

use server::run_server;
use tokio::signal;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting Blitz server...");
    
    let server = run_server();
    let shutdown = async {
        signal::ctrl_c().await.ok();
        info!("Shutdown signal received, stopping server...");
    };
    
    tokio::select! {
        res = server => {
            if let Err(e) = res {
                error!("Server error: {}", e);
            }
        }
        _ = shutdown => {}
    }
    
    info!("Server stopped");
    Ok(())
}
