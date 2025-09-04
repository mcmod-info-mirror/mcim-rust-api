use redis::{Client, aio::MultiplexedConnection};
use std::env;
use std::sync::Arc;

pub async fn connect()
-> Result<Arc<MultiplexedConnection>, Box<dyn std::error::Error + Send + Sync>> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let client = Client::open(redis_url)?;

    // 获取异步 MultiplexedConnection
    let conn = client.get_multiplexed_async_connection().await?;

    log::info!("Connected to Redis successfully!");

    Ok(Arc::new(conn))
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
