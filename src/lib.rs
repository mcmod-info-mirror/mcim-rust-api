pub mod config;
pub mod errors;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

use dotenvy::dotenv;
use std::env;

use crate::config::AppState;

pub fn build_app_state(
    mongo_client: mongodb::Client,
    redis_pool: std::sync::Arc<redis::aio::MultiplexedConnection>,
) -> AppState {
    dotenv().ok();
    AppState {
        db: mongo_client,
        redis_pool,
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

pub mod test_utils {
    use crate::build_app_state;
    use crate::routes::config;
    use actix_web::{web, App};
    use dotenvy::dotenv;
    use mongodb::{options::ClientOptions, Client};
    use redis::aio::MultiplexedConnection;
    use std::sync::Arc;

    pub async fn init_test_db() -> Client {
        let client_options = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .expect("Failed to parse MongoDB connection string");

        Client::with_options(client_options).expect("Failed to initialize MongoDB client")
    }

    pub async fn init_test_redis() -> Arc<MultiplexedConnection> {
        let client =
            redis::Client::open("redis://localhost:6379").expect("Failed to create Redis client");

        let pool = client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis");

        println!("Connected to Redis successfully!");
        Arc::new(pool)
    }

    pub async fn create_test_app() -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        dotenv().ok();

        let mongo_client = init_test_db().await;
        let redis_client = init_test_redis().await;

        App::new()
            .app_data(web::Data::new(build_app_state(mongo_client, redis_client)))
            .configure(config)
    }
}
