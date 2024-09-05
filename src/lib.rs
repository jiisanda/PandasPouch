pub mod client;
pub mod lru;
pub mod server;

pub mod pandas_pouch {
    tonic::include_proto!("pandas_pouch");
}