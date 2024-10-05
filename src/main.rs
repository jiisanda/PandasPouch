mod client;
mod config;
mod db;
mod lru;
mod server;
pub mod hash_ring;

use std::{sync::Arc, time::Duration};
use crate::server::pandas_pouch::NodeInfo;
use env_logger;
use hash_ring::{RingNodeInfo, HashRing};
use log::info;
use tokio::sync::Mutex;
use crate::config::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let settings = Settings::new()?;
    env_logger::init();

    let addr = format!("{}:{}", settings.local_addr, settings.local_port);
    let database_url = settings.database_url();

    // database initialization
    let db = db::Database::new(&database_url).await?;
    db.create_table_if_not_exists().await?;
    info!("Database table created or verified");

    let cache = Arc::new(Mutex::new(lru::LRUCache::new(100, Some(Duration::from_secs(60)))));

    let nodes = vec![
        // TODO! will update to be user input
        RingNodeInfo { host: "localhost".to_string(), port: 50051 },
        RingNodeInfo { host: "localhost".to_string(), port: 50052 },
    ];
    let hash_ring = Arc::new(Mutex::new(HashRing::new(nodes.clone(), 10)));

    let curr_node = RingNodeInfo {
        host: settings.local_addr.clone(),
        port: settings.local_port,
    };

    // convert Vec<RingNodeInfo> to Arc<Mutex<Vec<NodeInfo>>>
    let node_infos: Vec<NodeInfo> = nodes.into_iter().map(|node| NodeInfo {
        host: node.host,
        port: node.port as i32,
    }).collect();
    let nodes_arc = Arc::new(Mutex::new(node_infos));

    // starting the server
    server::run_server(&addr, db.into(), cache, hash_ring, curr_node, nodes_arc).await?;

    Ok(())
}
