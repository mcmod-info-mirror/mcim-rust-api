use std::{
    env,
    sync::Arc
};
use redis::aio::MultiplexedConnection;
use tokio_postgres::Client;

use crate::config::AppState;

pub fn create_app(
    pg_client: Arc<Client>,
    redis_pool: Arc<MultiplexedConnection>,
) -> AppState {
    AppState {
        db: pg_client,
        redis_pool: redis_pool,
        curseforge_api_url: env::var("CURSEFORGE_API_URL")
            .unwrap_or_else(|_| "https://api.curseforge.com".to_string()),
        modrinth_api_url: env::var("MODRINTH_API_URL")
            .unwrap_or_else(|_| "https://api.modrinth.com".to_string()),
        curseforge_api_key: env::var("CURSEFORGE_API_KEY").unwrap_or_else(|_| "".to_string()),
        curseforge_file_cdn_url: env::var("CURSEFORGE_FILE_CDN_URL")
            .unwrap_or_else(|_| "https://mediafilez.forgecdn.net".to_string()),
        modrinth_file_cdn_url: env::var("MODRINTH_FILE_CDN_URL")
            .unwrap_or_else(|_| "https://cdn.modrinth.com".to_string()),
    }
}