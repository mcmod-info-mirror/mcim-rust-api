pub mod database;
pub mod _redis;


#[derive(Clone)]
pub struct AppState {
    pub db: mongodb::Client,
    pub redis: redis::Client,
    pub curseforge_api_url: String,
    pub modrinth_api_url: String,
    pub curseforge_api_key: String,
}