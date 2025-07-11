use redis::aio::MultiplexedConnection;
use std::sync::Arc;
use sqlx::PgPool;
use std::env;

#[derive(Clone)]
pub struct AppState {
    pub mongodb: mongodb::Client,
    pub pgpool: PgPool,
    pub redis_pool: Arc<MultiplexedConnection>,
    pub curseforge_api_url: String,
    pub modrinth_api_url: String,
    pub curseforge_api_key: String,
    pub curseforge_file_cdn_url: String,
    pub modrinth_file_cdn_url: String,
}

pub fn build_app_state(
    mongo_client: mongodb::Client,
    pgpool: PgPool,
    redis_pool: Arc<redis::aio::MultiplexedConnection>,
) -> AppState {
    AppState {
        mongodb: mongo_client,
        pgpool: pgpool,
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
