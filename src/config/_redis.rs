use redis::{Client};
use std::env;

pub async fn connect() -> Result<Client, Box<dyn std::error::Error>> {
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let client = Client::open(redis_url)?;
    
    // 测试连接
    let mut con = client.get_connection()?;
    redis::cmd("PING").exec(&mut con).unwrap();
    
    println!("Connected to Redis successfully!");
    Ok(client)
}

pub fn get_redis_url() -> String {
    env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string())
}

pub fn get_redis_cache_database() -> i64 {
    env::var("REDIS_CACHE_DATABASE")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .unwrap_or(0)
}

pub fn get_redis_queue_database() -> i64 {
    env::var("REDIS_QUEUE_DATABASE")
        .unwrap_or_else(|_| "1".to_string())
        .parse()
        .unwrap_or(1)
}