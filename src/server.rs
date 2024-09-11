use std::sync::Arc;
use std::string::String;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tonic::transport::Server;

use pandas_pouch::pandas_pouch_cache_service_server::{PandasPouchCacheService, PandasPouchCacheServiceServer};
use pandas_pouch::{GetRequest, GetResponse, PutRequest, PutResponse};
use crate::db::Database;
use crate::lru::LRUCache;

pub mod pandas_pouch {
    tonic::include_proto!("pandas_pouch");
}

pub struct CacheServiceImpl {
    cache: Arc<Mutex<LRUCache<String, String>>>,
    db: Arc<Database>,
}

#[tonic::async_trait]
impl PandasPouchCacheService for CacheServiceImpl {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let key = request.into_inner().key;

        // getting the key, from the in-memory cache
        let mut cache = self.cache.lock().await;
        if let Some(value) = cache.get(&key) {
            return Ok(Response::new(GetResponse {
                found: true,
                value,
            }));
        }

        // if not in the memory, trying to get in the database
        match self.db.get(&key).await {
            Ok(Some(value)) => {
                // updating the in-memory cache
                cache.put(key.clone(), value.clone());
                Ok(Response::new(GetResponse {
                    found: true,
                    value,
                }))
            },
            Ok(None) => Ok(Response::new(GetResponse {
                found: false,
                value: String::new(),
            })),
            Err(e) => Err(Status::internal(format!("Database error: {}", e))),
        }
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        let req = request.into_inner();

        // update the in-memory cache
        let mut cache = self.cache.lock().await;
        cache.put(req.key.clone(), req.value.clone());

        // updating the database
        match self.db.put(&req.key, &req.value).await {
            Ok(_) => Ok(Response::new(PutResponse { success: true })),
            Err(e) => Err(Status::internal(format!("Database error:  {}", e))),
        }
    }
}

pub async fn run_server(addr: &str, database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cache = Arc::new(Mutex::new(LRUCache::new(10, None)));           // keeping capacity 10 for now
    let db: Arc<Database> = Arc::new(Database::new(database_url).await?);

    db.create_table_if_not_exists().await?;

    let service = CacheServiceImpl { cache, db };

    Server::builder()
        .add_service(PandasPouchCacheServiceServer::new(service))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
