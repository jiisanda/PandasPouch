mod client;
mod lru;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting server on 127.0.0.1:50051");
    server::run_server("127.0.0.1:50051").await?;
    
    Ok(())
}
