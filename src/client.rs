use tonic::transport::Channel;

use pandas_pouch::pandas_pouch_cache_service_client::PandasPouchCacheServiceClient;
use pandas_pouch::{GetRequest, PutRequest};

pub mod pandas_pouch {
    tonic::include_proto!("pandas_pouch");
}

#[allow(dead_code)]
pub struct Client {
    client: PandasPouchCacheServiceClient<Channel>,
}

#[allow(dead_code)]
impl Client {
    pub async fn new(host: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let addr = format!("http://{}:{}", host, port);
        let channel = Channel::from_shared(addr)?.connect().await?;
        let client = PandasPouchCacheServiceClient::new(channel);
        Ok(Client { client })
    }

    pub async fn get(&mut self, key: String) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(GetRequest { key });
        let response = self.client.get(request).await?.into_inner();
        if response.found {
            Ok(Some(response.value))
        } else {
            Ok(None)
        }
    }

    pub async fn put(&mut self, key: String, value: String) -> Result<bool, Box<dyn std::error::Error>> {
        let request = tonic::Request::new(PutRequest { key, value });
        let response = self.client.put(request).await?.into_inner();
        Ok(response.success)
    }
}
