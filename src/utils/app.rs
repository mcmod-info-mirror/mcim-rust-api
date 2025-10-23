use redis::aio::MultiplexedConnection;
use reqwest::Client;
use std::env;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: mongodb::Client,
    pub redis_pool: Arc<MultiplexedConnection>,
    pub curseforge_api_url: String,
    pub modrinth_api_url: String,
    pub curseforge_api_key: String,
    pub file_cdn_enabled: bool,
    pub curseforge_file_cdn_url: String,
    pub curseforge_file_cdn_fallback_url: String,
    pub curseforge_cdn_primary_percentage: u8,
    pub modrinth_file_cdn_url: String,
    pub modrinth_file_cdn_fallback_url: String,
    pub modrinth_cdn_primary_percentage: u8,
    pub curseforge_avatar_cdn_url: String,
    pub curseforge_avatar_cdn_fallback_url: String,
    pub curseforge_avatar_cdn_primary_percentage: u8,
    pub modrinth_avatar_cdn_url: String,
    pub modrinth_avatar_cdn_fallback_url: String,
    pub modrinth_avatar_cdn_primary_percentage: u8,
    pub http_client: Client, // 共享的 HTTP 客户端
}

pub fn build_app_state(
    mongo_client: mongodb::Client,
    redis_pool: std::sync::Arc<redis::aio::MultiplexedConnection>,
) -> AppState {
    AppState {
        db: mongo_client,
        redis_pool,
        curseforge_api_url: env::var("CURSEFORGE_API_URL")
            .unwrap_or_else(|_| "https://api.curseforge.com".to_string()),
        modrinth_api_url: env::var("MODRINTH_API_URL")
            .unwrap_or_else(|_| "https://api.modrinth.com".to_string()),
        curseforge_api_key: env::var("CURSEFORGE_API_KEY").unwrap_or_else(|_| "".to_string()),
        file_cdn_enabled: env::var("FILE_CDN_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true),
        curseforge_file_cdn_url: env::var("CURSEFORGE_FILE_CDN_URL")
            .unwrap_or_else(|_| "https://mediafilez.forgecdn.net".to_string()),
        modrinth_file_cdn_url: env::var("MODRINTH_FILE_CDN_URL")
            .unwrap_or_else(|_| "https://cdn.modrinth.com".to_string()),
        curseforge_file_cdn_fallback_url: env::var("CURSEFORGE_FILE_CDN_FALLBACK_URL")
            .unwrap_or_else(|_| "https://mediafilez.forgecdn.net".to_string()),
        curseforge_cdn_primary_percentage: env::var("CURSEFORGE_CDN_PRIMARY_PERCENTAGE")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u8>()
            .unwrap_or(100),
        modrinth_file_cdn_fallback_url: env::var("MODRINTH_FILE_CDN_FALLBACK_URL")
            .unwrap_or_else(|_| "https://cdn.modrinth.com".to_string()),
        modrinth_cdn_primary_percentage: env::var("MODRINTH_CDN_PRIMARY_PERCENTAGE")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u8>()
            .unwrap_or(100),
        curseforge_avatar_cdn_url: env::var("CURSEFORGE_AVATAR_CDN_URL")
            .unwrap_or_else(|_| "https://cdn.modrinth.com".to_string()),
        curseforge_avatar_cdn_fallback_url: env::var("CURSEFORGE_AVATAR_CDN_FALLBACK_URL")
            .unwrap_or_else(|_| "https://cdn.modrinth.com".to_string()),
        curseforge_avatar_cdn_primary_percentage: env::var(
            "CURSEFORGE_AVATAR_CDN_PRIMARY_PERCENTAGE",
        )
        .unwrap_or_else(|_| "100".to_string())
        .parse::<u8>()
        .unwrap_or(100),
        modrinth_avatar_cdn_url: env::var("MODRINTH_AVATAR_CDN_URL")
            .unwrap_or_else(|_| "https://media.forgecdn.net".to_string()),
        modrinth_avatar_cdn_fallback_url: env::var("MODRINTH_AVATAR_CDN_FALLBACK_URL")
            .unwrap_or_else(|_| "https://media.forgecdn.net".to_string()),
        modrinth_avatar_cdn_primary_percentage: env::var("MODRINTH_AVATAR_CDN_PRIMARY_PERCENTAGE")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u8>()
            .unwrap_or(100),
        http_client: Client::new(), // 创建一个共享的 HTTP 客户端
    }
}
