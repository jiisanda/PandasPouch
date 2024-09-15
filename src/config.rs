use std::env;
use config::{Config, ConfigError, Environment, File};
use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub local_addr: String,
    pub local_port: u16,
    pub database: DatabaseSettings,
    pub rust_log: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub username: String,
    pub password: String,
    pub name: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("APP"))
            .build()?;

        let settings: Settings = s.try_deserialize()?;
        env::set_var("RUST_LOG", &settings.rust_log);

        info!("connected to database: {:?}", settings.database_url());
        
        Ok(settings)
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}/{}",
            self.database.username, self.database.password, self.database.host, self.database.name
        )
    }
}
