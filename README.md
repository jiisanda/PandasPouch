![banner](static/banner.png)

# 🐼 pandas-pouch 🐼

A Distributed Caching Service with Rust 🦀🦀.


## Progress

- ✅ Basic Cache
- ✅ LRU Cache
- ✅ Database Setup
- ✅ API Integration
- ✅ Docker Setup
- 🟡 Consistent Hashing
- 🟡 Distributed Cache - Dedicated cache cluster

## Pre-requisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://docs.docker.com/get-docker/)
- gRPC
  - [grpcurl](https://github.com/fullstorydev/grpcurl)

## How can pandas-pouch be used with current features?

### Configuration

Create a `config` directory. And add `default.toml` file with the following configuration.
Sample default.toml file is provided in the `config` directory [config/sample_default.toml](config/sample_default.toml). Also add a `.env` file with the 
configuration in [config/.env.sample](config/.env.sample) file.

### Running the Service

1. Clone the repository and navigate to the project directory.

```bash
git clone https://github.com/jiisanda/pandas-pouch.git
cd pandas-pouch
```

2. Start the service using `docker-compose`.

```bash
docker-compose build
docker-compose up
```

### Interacting with the Service

You can use `grpcurl` to interact with the service. Install `grpcurl` using the installation guide in the [grpcurl repository](https://github.com/fullstorydev/grpcurl)

Run the following command to interact with the service.

1. Put operation
```bash
grpcurl -plaintext -proto proto/cache.proto -d '{"key": "key2", "value": "value2"}' 0.0.0.0:50051 pandas_pouch.PandasPouchCacheService/Put
```

2. Get Operation
```bash
grpcurl -plaintext -proto proto/cache.proto -d '{"key": "key2"}' 0.0.0.0:50051 pandas_pouch.PandasPouchCacheService/Get
```

3. PrintAll Operation
```bash
grpcurl -plaintext -proto proto/cache.proto 0.0.0.0:50051 pandas_pouch.PandasPouchCacheService/PrintAll
```

### pandas-pouch as a crate

To use pandas-pouch as a crate, add the following to your `Cargo.toml` file.

```toml
[dependencies]
pandas-pouch = { git = "https://github.com/jiisanda/pandas-pouch.git" }
```

Using it in code:
```rust
use pandas_pouch::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let mut client = Client::new("http://localhost:50051").await?;
  
  // put a value
  client.put("key1".to_string(), "value1".to_string()).await?;
  
  // get a value
  let value = client.get("key1".to_string()).await?;
  println!("Value: {:?}", value);
  
  Ok(())
}
```

## License

This project is licensed under the [MIT License](LICENSE).
