use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub services: ServicesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_body_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub consensus_service_url: Option<String>,
    pub blockchain_service_url: Option<String>,
    pub storage_service_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_body_size: 10 * 1024 * 1024, // 10MB
            },
            services: ServicesConfig {
                consensus_service_url: None,
                blockchain_service_url: None,
                storage_service_url: None,
            },
        }
    }
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Try to load from config file first
    if let Ok(content) = fs::read_to_string("config.toml") {
        let config: Config = toml::from_str(&content)?;
        return Ok(config);
    }

    // Try to load from .env and return default with environment overrides
    dotenv::ok();
    let config = Config {
        server: ServerConfig {
            host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            max_body_size: std::env::var("MAX_BODY_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10 * 1024 * 1024),
        },
        services: ServicesConfig {
            consensus_service_url: std::env::var("CONSENSUS_SERVICE_URL").ok(),
            blockchain_service_url: std::env::var("BLOCKCHAIN_SERVICE_URL").ok(),
            storage_service_url: std::env::var("STORAGE_SERVICE_URL").ok(),
        },
    };

    Ok(config)
}
