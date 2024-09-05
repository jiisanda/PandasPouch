use tokio::time::{sleep, Duration};
use pandas_pouch::client::Client;
use pandas_pouch::server;

#[tokio::test]
async fn test_server_client() {
    tokio::spawn(async {
        if let Err(e) = server::run_server("127.0.0.1:50052").await {
            println!("Server Error: {}", e);
        }
    });

    sleep(Duration::from_secs(1)).await;
    let mut client = Client::new("127.0.0.1", 50052).await.unwrap();

    client.put("test_key".to_string(), "test_value".to_string()).await.unwrap();
    let value = client.get("test_key".to_string()).await.unwrap();
    assert_eq!(value, Some("test_value".to_string()));
    
    let value = client.get("non_existent_key".to_string()).await.unwrap();
    assert_eq!(value, None);
}