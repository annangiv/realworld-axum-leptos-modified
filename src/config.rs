use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub api_url: String,
    pub environment: Environment,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    pub fn global() -> &'static Config {
        CONFIG.get().expect("Config not initialized")
    }

    pub fn init() -> Result<(), String> {
        let config = Config {
            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            api_url: std::env::var("API_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
            environment: match std::env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string())
                .as_str() 
            {
                "production" => Environment::Production,
                _ => Environment::Development,
            },
        };

        CONFIG.set(config)
            .map_err(|_| "Config already initialized".to_string())
    }
}