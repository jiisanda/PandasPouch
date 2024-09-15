use std::sync::Arc;
use std::string::String;
use log::{debug, error, info};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tonic::transport::Server;

use pandas_pouch::pandas_pouch_cache_service_server::{PandasPouchCacheService, PandasPouchCacheServiceServer};
use pandas_pouch::{GetRequest, GetResponse, PutRequest, PutResponse, PrintAllRequest, PrintAllResponse, KeyValuePair};
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
        info!("GET: key: {}", key);

        // getting the key, from the in-memory cache
        let mut cache = self.cache.lock().await;
        if let Some(value) = cache.get(&key) {
            debug!("Cache hit for key: {}", key);
            return Ok(Response::new(GetResponse {
                found: true,
                value,
            }));
        }

        // if not in the memory, trying to get in the database
        debug!("Cache miss for key: {}", key);
        match self.db.get(&key).await {
            Ok(Some(value)) => {
                // updating the in-memory cache
                debug!("Found value in database for key: {}", key);
                cache.put(key.clone(), value.clone());
                Ok(Response::new(GetResponse {
                    found: true,
                    value,
                }))
            },
            Ok(None) => {
                info!("Key not found in cache or database: {}", key);
                Ok(Response::new(GetResponse {
                    found: false,
                    value: String::new(),
                }))
            },
            Err(e) => {
                error!("Database error while getting key {}: {}", key, e);
                Err(Status::internal(format!("Database error: {}", e)))
            },
        }
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        let req = request.into_inner();
        info!("PUT: {}", req.key);

        // update the in-memory cache
        let mut cache = self.cache.lock().await;
        cache.put(req.key.clone(), req.value.clone());

        // updating the database
        match self.db.put(&req.key, &req.value).await {
            Ok(_) => {
                debug!("Successfully put key-value pair in cache and database");
                Ok(Response::new(PutResponse { success: true })) 
            },
            Err(e) => {
                error!("Database error while putting key {}: {}", req.key, e);
                Err(Status::internal(format!("Database error:  {}", e))) 
            },
        }
    }

    async fn print_all(&self, _request: Request<PrintAllRequest>) -> Result<Response<PrintAllResponse>, Status> {
        info!("Received PrintAll request");
        let mut cache = self.cache.lock().await;
        let pairs = cache.print().into_iter().map(|(k, v)| {
            debug!("Printing cache entry: {} -> {}", k, v);
            KeyValuePair {
                key: k,
                value: v,
            }
        }).collect::<Vec<_>>();

        info!("Returning {} cache entries", pairs.len());
        Ok(Response::new(PrintAllResponse { pairs }))
    }
}

pub async fn run_server(addr: &str, database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing server with address: {}", addr);
    let cache = Arc::new(Mutex::new(LRUCache::new(10, None)));          // keeping capacity 10 for now
    let db: Arc<Database> = Arc::new(Database::new(database_url).await?);

    db.create_table_if_not_exists().await?;
    info!("Database table created or verified");

    let service = CacheServiceImpl { cache, db };

    info!("Starting server on {}", addr);
    Server::builder()
        .add_service(PandasPouchCacheServiceServer::new(service))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
