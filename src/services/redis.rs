use std::error::Error;
use redis::{Client, RedisError};

#[derive(Clone)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn check_connection(&self) -> Result<(), RedisError> {
        let mut conn = self.client.get_connection()?;
        let result: String = redis::cmd("PING")
            .query(&mut conn)?;
        
        if result == "PONG" {
            Ok(())
        } else {
            Err(RedisError::from((redis::ErrorKind::ResponseError, "Unexpected response")))
        }
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), RedisError> {
        let mut conn = self.client.get_connection()?;
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query(&mut conn)
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, RedisError> {
        let mut conn = self.client.get_connection()?;
        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query(&mut conn)?;
        Ok(result)
    }

    pub async fn delete(&self, key: &str) -> Result<bool, RedisError> {
        let mut conn = self.client.get_connection()?;
        let result: i32 = redis::cmd("DEL")
            .arg(key)
            .query(&mut conn)?;
        Ok(result > 0)
    }
}