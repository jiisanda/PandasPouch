use pandas_pouch::client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new("127.0.0.1", 50051).await?;

    client.put("key1".to_string(), "value1".to_string()).await?;
    println!("Put key!: value1");
    
    let value = client.get("key1".to_string()).await?;
    println!("Got key1: {:?}", value);
    
    let value = client.get("key2".to_string()).await?;
    println!("Got key2: {:?}", value);
    
    Ok(())
}
