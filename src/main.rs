mod client;
mod config;
mod db;
mod lru;
mod server;
pub mod hash_ring;
use env_logger;
use log::info;
use crate::config::Settings;
// use crate::hash_ring::NodeInfo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let settings = Settings::new()?;
    env_logger::init();

    let addr = format!("{}:{}", settings.local_addr, settings.local_port);
    let database_url = settings.database_url();

    // let self_node_info = NodeInfo { host: "localhost", port: 50051 };

    info!("Starting server on {}", addr);
    server::run_server(&addr, &database_url).await?;

    Ok(())
}
