mod client;
mod config;
mod db;
mod lru;
mod server;

use crate::config::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let settings = Settings::new()?;

    let addr = format!("{}:{}", settings.local_addr, settings.local_port);
    let database_url = settings.database_url();

    println!("Starting server on {}", addr);
    server::run_server(&addr, &database_url).await?;

    Ok(())
}
