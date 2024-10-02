use std::sync::Arc;
use std::string::String;
use log::{debug, error, info};
use tokio::sync::Mutex;
use tonic::{async_trait, Request, Response, Status};
use tonic::transport::Server;

use pandas_pouch::pandas_pouch_cache_service_server::{
    PandasPouchCacheService,
    PandasPouchCacheServiceServer,
};
use pandas_pouch::pandas_pouch_cache_service_client::PandasPouchCacheServiceClient;
use pandas_pouch::{
    GetRequest,
    GetResponse,
    PutRequest,
    PutResponse,
    PrintAllRequest,
    PrintAllResponse,
    KeyValuePair,
    JoinClusterRequest,
    JoinClusterResponse,
    LeaveClusterRequest,
    LeaveClusterResponse,
    NodeInfo,
};
use crate::db::Database;
use crate::lru::LRUCache;

pub mod pandas_pouch {
    tonic::include_proto!("pandas_pouch");
}

pub struct CacheServiceImpl {
    cache: Arc<Mutex<LRUCache<String, String>>>,
    db: Arc<Database>,
    nodes: Arc<Mutex<Vec<NodeInfo>>>,
}

#[async_trait]
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

    async fn forward_get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let key = request.into_inner().key;
        info!("Forwarding GET request for key: {}", key);

        // forward request to another node in cluster
        let node = self.get_next_node().await?;
        let mut client = PandasPouchCacheServiceClient::connect(node).await.map_err(|e| {
            error!("Failed to connect to node: {}", e);
            Status::internal("Failed to connect to node")
        })?;

        client.get(Request::new(GetRequest { key })).await.map_err(|e| {
            error!("Failed to forward GET request: {}", e);
            Status::internal("Failed to forward GET request")
        })
    }

    async fn forward_put(&self, request: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        let req = request.into_inner();
        info!("Forwarding PUT request for key: {}", req.key);

        // forward request to another node in cluster
        let node = self.get_next_node().await?;
        let mut client = PandasPouchCacheServiceClient::connect(node).await.map_err(|e| {
            error!("Failed to connect to node: {}", e);
            Status::internal("Failed to connect to node")
        })?;

        client.put(Request::new(PutRequest { key: req.key, value: req.value })).await.map_err(|e| {
            error!("Failed to forward PUT request: {}", e);
            Status::internal("Failed to forward PUT request")
        })
    }

    async fn join_cluster(&self, request: Request<JoinClusterRequest>) -> Result<Response<JoinClusterResponse>, Status> {
        let node_info = request.into_inner().joining_node;
        info!("Node joining cluster: {:?}", node_info);

        // add joining node to cluster
        let mut nodes = self.nodes.lock().await;
        nodes.push(node_info.expect("NodeInfo should not be None"));

        Ok(Response::new(JoinClusterResponse {
            success: true,
            current_nodes: nodes.clone(),
        }))
    }

    async fn leave_cluster(&self, request: Request<LeaveClusterRequest>) -> Result<Response<LeaveClusterResponse>, Status> {
        let node_info = request.into_inner().leaving_node;
        info!("Node leaving cluster: {:?}", node_info);

        // remove the leaving node from cluster
        let mut nodes = self.nodes.lock().await;
        nodes.retain(|node| node != node_info.as_ref().expect("NodeInfo should not be None"));

        Ok(Response::new(LeaveClusterResponse {
            success: true,
        }))
    }
}

impl CacheServiceImpl {
    async fn get_next_node(&self) -> Result<String, Status> {
        let nodes = self.nodes.lock().await;
        if nodes.is_empty() {
            return Err(Status::internal("No nodes available in the cluster"));
        }

        // for now, returning the first node, will integrate with hash_ring
        Ok(format!("http://{}:{}", nodes[0].host, nodes[0].port))
    }
}

pub async fn run_server(addr: &str, database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing server with address: {}", addr);
    let cache = Arc::new(Mutex::new(LRUCache::new(10, None)));          // keeping capacity 10 for now
    let db: Arc<Database> = Arc::new(Database::new(database_url).await?);
    let nodes = Arc::new(Mutex::new(Vec::new()));           // list of nodes in the cluster

    db.create_table_if_not_exists().await?;
    info!("Database table created or verified");

    let service = CacheServiceImpl { cache, db, nodes };

    info!("Starting server on {}", addr);
    Server::builder()
        .add_service(PandasPouchCacheServiceServer::new(service))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
