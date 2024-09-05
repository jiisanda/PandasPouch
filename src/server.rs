use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use crate::lru::LRUCache;

use pandas_pouch::pandas_pouch_cache_service_server::{PandasPouchCacheService, PandasPouchCacheServiceServer};
use pandas_pouch::{GetRequest, GetResponse, PutRequest, PutResponse};


pub mod pandas_pouch {
    tonic::include_proto!("pandas_pouch");
}

pub struct CacheServiceImpl {
    cache: Arc<Mutex<LRUCache<String, String>>>,
}

#[tonic::async_trait]
impl PandasPouchCacheService for CacheServiceImpl {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let key = request.into_inner().key;
        let mut cache = self.cache.lock().await;
        let result = cache.get(&key);

        Ok(Response::new(GetResponse {
            found: result.is_some(),
            value: result.unwrap_or_default(),
        }))
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        let req = request.into_inner();
        let mut cache = self.cache.lock().await;
        cache.put(req.key, req.value);

        Ok(Response::new(PutResponse { success: true }))
    }
}

pub async fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache = Arc::new(Mutex::new(LRUCache::new(10, None)));           // keeping capacity 10 for now
    let service = CacheServiceImpl { cache };

    Server::builder()
        .add_service(PandasPouchCacheServiceServer::new(service))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
