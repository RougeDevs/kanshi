use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::error::Error;
use tokio_postgres::{Client as PgClient, NoTls};

use crate::config::Config;

use super::redis::RedisClient;

#[async_trait]
pub trait DataStorage: Send + Sync {
    async fn store_json(&self, key: &str, value: Value) -> Result<(), Box<dyn Error>>;
    async fn retrieve_json(&self, key: &str) -> Result<Option<Value>, Box<dyn Error>>;
    async fn delete(&self, key: &str) -> Result<bool, Box<dyn Error>>;
}

#[async_trait]
pub trait TypedStorage {
    async fn store<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<(), Box<dyn Error>>;
    async fn retrieve<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> Result<Option<T>, Box<dyn Error>>;
}

#[async_trait]
impl<S: DataStorage + Send + Sync> TypedStorage for S {
    async fn store<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<(), Box<dyn Error>> {
        let json_value = serde_json::to_value(value)?;
        self.store_json(key, json_value).await
    }

    async fn retrieve<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> Result<Option<T>, Box<dyn Error>> {
        if let Some(value) = self.retrieve_json(key).await? {
            Ok(Some(serde_json::from_value(value)?))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone)]
pub struct RedisStorage {
    client: RedisClient,
}

impl RedisStorage {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn Error>> {
        let client = RedisClient::new(redis_url)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl DataStorage for RedisStorage {
    async fn store_json(&self, key: &str, value: Value) -> Result<(), Box<dyn Error>> {
        let serialized = value.to_string();
        self.client.set(key, &serialized).await?;
        Ok(())
    }

    async fn retrieve_json(&self, key: &str) -> Result<Option<Value>, Box<dyn Error>> {
        if let Some(value) = self.client.get(key).await? {
            Ok(Some(serde_json::from_str(&value)?))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, key: &str) -> Result<bool, Box<dyn Error>> {
        Ok(self.client.delete(key).await?)
    }
}

// PostgreSQL implementation
pub struct PostgresStorage {
    client: PgClient,
}

impl PostgresStorage {
    pub async fn new(config: &str) -> Result<Self, Box<dyn Error>> {
        let (client, connection) = tokio_postgres::connect(config, NoTls).await?;
        
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL connection error: {}", e);
            }
        });

        client
            .execute(
                "CREATE TABLE IF NOT EXISTS key_value_store (
                    key TEXT PRIMARY KEY,
                    value JSONB NOT NULL
                )",
                &[],
            )
            .await?;

        Ok(Self { client })
    }
}

#[async_trait]
impl DataStorage for PostgresStorage {
    async fn store_json(&self, key: &str, value: Value) -> Result<(), Box<dyn Error>> {
        let json_str = value.to_string();
        self.client
            .execute(
                "INSERT INTO key_value_store (key, value) 
                 VALUES ($1, $2::jsonb) 
                 ON CONFLICT (key) DO UPDATE SET value = $2::jsonb",
                &[&key, &json_str],
            )
            .await?;
        Ok(())
    }

    async fn retrieve_json(&self, key: &str) -> Result<Option<Value>, Box<dyn Error>> {
        let row = self.client
            .query_opt(
                "SELECT value::text FROM key_value_store WHERE key = $1",
                &[&key],
            )
            .await?;

        if let Some(row) = row {
            let json_str: String = row.get(0);
            Ok(Some(serde_json::from_str(&json_str)?))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, key: &str) -> Result<bool, Box<dyn Error>> {
        let result = self.client
            .execute(
                "DELETE FROM key_value_store WHERE key = $1",
                &[&key],
            )
            .await?;
        Ok(result > 0)
    }
}

pub struct StorageManager {
    storage: Box<dyn DataStorage>,
}

impl StorageManager {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let storage: Box<dyn DataStorage> = match config.storage_url.starts_with("postgres") {
            true => Box::new(PostgresStorage::new(&config.storage_url).await?),
            false => Box::new(RedisStorage::new(&config.storage_url)?),
        };
        Ok(Self { storage })
    }
}

#[async_trait]
impl TypedStorage for StorageManager {
    async fn store<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<(), Box<dyn Error>> {
        let json_value = serde_json::to_value(value)?;
        self.storage.store_json(key, json_value).await
    }

    async fn retrieve<T: DeserializeOwned + Send + Sync>(&self, key: &str) -> Result<Option<T>, Box<dyn Error>> {
        if let Some(value) = self.storage.retrieve_json(key).await? {
            Ok(Some(serde_json::from_value(value)?))
        } else {
            Ok(None)
        }
    }
}

impl StorageManager {
    pub async fn delete(&self, key: &str) -> Result<bool, Box<dyn Error>> {
        self.storage.delete(key).await
    }
}