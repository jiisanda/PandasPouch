use std::sync::Arc;
use std::string::String;
use std::fmt;
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
use crate::hash_ring::{HashRing, RingNodeInfo};
use crate::lru::LRUCache;

pub mod pandas_pouch {
    tonic::include_proto!("pandas_pouch");

    use super::fmt;

    impl fmt::Display for NodeInfo {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}:{}", self.host, self.port)
        }
    }
}

pub struct CacheServiceImpl {
    cache: Arc<Mutex<LRUCache<String, String>>>,
    db: Arc<Database>,
    hash_ring: Arc<Mutex<HashRing<RingNodeInfo>>>,
    curr_node: RingNodeInfo,
    nodes: Arc<Mutex<Vec<NodeInfo>>>,
}

#[async_trait]
impl PandasPouchCacheService for CacheServiceImpl {
    async fn get(&self, request: Request<GetRequest>) -> Result<Response<GetResponse>, Status> {
        let key = request.into_inner().key;
        info!("GET: key: {}", key);

        let hash_ring = self.hash_ring.lock().await;
        let node = hash_ring.get_node(key.clone()).ok_or(Status::not_found("Node not found"))?;

        // check if curr_node is responsible, if yes get value form cache
        if node.host == self.curr_node.host && node.port == self.curr_node.port {
            let mut cache = self.cache.lock().await;
            if let Some(value) = cache.get(&key) {
                debug!("Cache hit for key: {}", key);
                return Ok(Response::new(GetResponse {
                    found: true,
                    value,
                }));
            }

            // if not in the cache, trying to get in the database
            debug!("Cache miss for key: {}", key);
            match self.db.get(&key).await {
                Ok(Some(value)) => {
                    // updating the in-memory cache
                    debug!("Found value in database for key: {}", key);
                    cache.put(key.clone(), value.clone());
                    return Ok(Response::new(GetResponse {
                        found: true,
                        value,
                    }));
                },
                Ok(None) => {
                    info!("Key not found in cache or database: {}", key);
                    return Ok(Response::new(GetResponse {
                        found: false,
                        value: String::new(),
                    }));
                },
                Err(e) => {
                    error!("Database error while getting key {}: {}", key, e);
                    return Err(Status::internal(format!("Database error: {}", e)));
                },
            }
        }

        // forward request to appropriate node
        let mut client = PandasPouchCacheServiceClient::connect(format!("http://{}:{}", node.host, node.port))
            .await
            .map_err(|e| Status::internal(format!("Failed to connect to node: {}", e)))?;
        let response = client.get(Request::new(GetRequest { key })).await?;
        Ok(response)
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        let req = request.into_inner();
        info!("PUT: {}", req.key);

        let hash_ring = self.hash_ring.lock().await;
        let node = hash_ring.get_node(req.key.clone()).ok_or(Status::not_found("Node not found"))?;

        // check if curr_node is responsible, if yes put the value in the cache and database
        if node.host == self.curr_node.host && node.port == self.curr_node.port {
            let mut cache = self.cache.lock().await;
            cache.put(req.key.clone(), req.value.clone());

            match self.db.put(&req.key, &req.value).await {
                Ok(_) => {
                    debug!("Successfully put key-value pair in cache and database");
                    return Ok(Response::new(PutResponse { success: true }));
                },
                Err(e) => {
                    error!("Database error while putting key {}: {}", req.key, e);
                    return Err(Status::internal(format!("Database error:  {}", e)));
                },
            };
        }

        // forward the request to appropriate node
        let mut client = PandasPouchCacheServiceClient::connect(format!("http://{}:{}", node.host, node.port))
            .await
            .map_err(|e| Status::internal(format!("Failed to connect to node: {}", e)))?;
        let response = client.put(Request::new(PutRequest {
            key: req.key,
            value: req.value,
        })).await?;
        Ok(response)
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
        let node = self.get_next_node(&key).await?;
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
        let node = self.get_next_node(&req.key).await?;
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
        let mut hash_ring = self.hash_ring.lock().await;
        if let Some(node) = node_info {
            nodes.push(node.clone());
            let ring_node = RingNodeInfo { host: node.host, port: node.port as u16 };
            hash_ring.add_node(&ring_node);
        }

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
        let mut hash_ring = self.hash_ring.lock().await;
        if let Some(node) = node_info {
            nodes.retain(|n| n != &node);
            let ring_node = RingNodeInfo { host: node.host, port: node.port as u16 };
            hash_ring.remove_node(&ring_node);
        }

        Ok(Response::new(LeaveClusterResponse {
            success: true,
        }))
    }
}

impl CacheServiceImpl {
    async fn get_next_node(&self, key: &str) -> Result<String, Status> {
        let hash_ring = self.hash_ring.lock().await;
        if let Some(node) = hash_ring.get_node(key.to_string()) {
            Ok(format!("http://{}:{}", node.host, node.port))
        } else {
            Err(Status::internal("No nodes available in the cluster"))
        }
    }
}

pub async fn run_server(
    addr: &str,
    db:Arc<Database>,
    cache: Arc<Mutex<LRUCache<String, String>>>,
    hash_ring: Arc<Mutex<HashRing<RingNodeInfo>>>,
    curr_node: RingNodeInfo,
    nodes: Arc<Mutex<Vec<NodeInfo>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = CacheServiceImpl { cache, db, hash_ring, curr_node, nodes };

    info!("Starting server on {}", addr);
    Server::builder()
        .add_service(PandasPouchCacheServiceServer::new(service))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
