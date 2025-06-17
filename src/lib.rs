pub mod config;
pub mod models;
pub mod routes;
pub mod services;
pub mod errors;
pub mod utils;

pub mod test_utils {
    use actix_web::{web, App};
    use mongodb::{options::ClientOptions, Client};
    use redis::aio::MultiplexedConnection;
    use std::sync::Arc;
    use std::env;
    use crate::routes::config;
    use crate::config::AppState;

    pub async fn init_test_db() -> Client {
        let client_options = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .expect("Failed to parse MongoDB connection string");

        Client::with_options(client_options).expect("Failed to initialize MongoDB client")
    }

    pub async fn init_test_redis() -> Arc<MultiplexedConnection> {
        let client = redis::Client::open("redis://localhost:6379")
            .expect("Failed to create Redis client");

        let pool = client.get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis");

        println!("Connected to Redis successfully!");
        Arc::new(pool)
    }

    pub async fn create_test_app() ->  App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        let mongo_client = init_test_db().await;
        let redis_client = init_test_redis().await;
        let app_state = AppState {
            db: mongo_client,
            redis_pool: redis_client,
            curseforge_api_url: "https://api.curseforge.com".to_string(),
            modrinth_api_url: "https://api.modrinth.com".to_string(),
            curseforge_api_key: env::var("CURSEFORGE_API_KEY").unwrap_or_else(|_| "".to_string()),
        };

        App::new()
                .app_data(web::Data::new(app_state))
                .configure(|cfg| {
                    config(cfg);
                })
    }
}