pub mod _redis;
pub mod database;

use redis::aio::MultiplexedConnection;
use tokio_postgres::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Client>,
    pub redis_pool: Arc<MultiplexedConnection>,
    pub curseforge_api_url: String,
    pub modrinth_api_url: String,
    pub curseforge_api_key: String,
    pub curseforge_file_cdn_url: String,
    pub modrinth_file_cdn_url: String,
}
