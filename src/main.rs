pub mod config;
pub mod errors;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

use crate::config::_redis::connect as connect_redis;
use crate::config::database::connect as connect_mongo;
use crate::config::AppState;
use crate::errors::ApiError;
use crate::routes::config as routes_config;

#[utoipauto]
#[derive(OpenApi)]
#[openapi(info(
    title = "MCIM API",
    version = "1.0.0",
    contact(
        name = "mcmod-info-mirror",
        // email = "z0z0r4@outlook.com",
        url = "https://github.com/mcmod-info-mirror"
    )
))]
pub struct OpenApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // 配置MongoDB连接
    let mongo_client = connect_mongo().await.expect("Failed to connect to MongoDB");
    let redis_pool = connect_redis().await.expect("Failed to connect to Redis");

    // 获取服务器端口，默认为 8080
    let port = env::var("PORT").unwrap_or_else(|_| "28080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    let redis_pool_clone = redis_pool.clone();
    let app = move || {
        let app_data = web::Data::new(AppState {
            db: mongo_client.clone(),
            redis_pool: redis_pool_clone.clone(),
            curseforge_api_url: env::var("CURSEFORGE_API_URL")
                .unwrap_or_else(|_| "https://api.curseforge.com".to_string()),
            modrinth_api_url: env::var("MODRINTH_API_URL")
                .unwrap_or_else(|_| "https://api.modrinth.com".to_string()),
            curseforge_api_key: env::var("CURSEFORGE_API_KEY").unwrap_or_else(|_| "".to_string()),
        });

        App::new()
            .app_data(app_data)
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _| ApiError::BadRequest(err.to_string()).into()),
            )
            .app_data(
                web::QueryConfig::default()
                    .error_handler(|err, _| ApiError::BadRequest(err.to_string()).into()),
            )
            .app_data(
                web::PathConfig::default()
                    .error_handler(|err, _| ApiError::BadRequest(err.to_string()).into()),
            )
            .configure(routes_config)
            .service(SwaggerUi::new("/docs/{_:.*}").url("/openapi.json", OpenApiDoc::openapi()))
    };

    HttpServer::new(app).bind(&bind_address)?.run().await
}
