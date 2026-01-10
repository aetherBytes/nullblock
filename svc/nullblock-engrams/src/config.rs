use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub service_name: String,
    pub database_url: String,
    pub erebus_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            service_name: env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "nullblock-engrams".to_string()),
            database_url: env::var("ENGRAMS_DATABASE_URL")
                .or_else(|_| env::var("DATABASE_URL"))
                .unwrap_or_else(|_| {
                    "postgresql://postgres:postgres_secure_pass@localhost:5441/agents".to_string()
                }),
            erebus_url: env::var("EREBUS_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        })
    }
}
