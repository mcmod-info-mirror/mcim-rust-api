pub mod database;
pub mod _redis;

use std::sync::Arc;
use redis::aio::MultiplexedConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: mongodb::Client,
    pub redis_pool: Arc<MultiplexedConnection>,
    pub curseforge_api_url: String,
    pub modrinth_api_url: String,
    pub curseforge_api_key: String,
}