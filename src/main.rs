pub mod config;
pub mod errors;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use std::env;
use utoipa::OpenApi;
use utoipauto::utoipauto;

use crate::config::_redis::connect as connect_redis;
use crate::config::database::connect as connect_mongo;
use crate::config::AppState;
use crate::errors::ApiError;
use crate::routes::config as routes_config;

#[allow(unknown_lints)]
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

async fn serve_openapi() -> impl Responder {
    let openapi_string = OpenApiDoc::openapi()
        .to_json()
        .expect("Should serialize to JSON");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(openapi_string.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 初始化环境变量
    dotenv().ok();

    // 初始化日志记录
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // 配置MongoDB连接
    let mongo_client = connect_mongo().await.expect("Failed to connect to MongoDB");
    let redis_pool = connect_redis().await.expect("Failed to connect to Redis");

    // 获取服务器端口，默认为 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
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
            curseforge_file_cdn_url: env::var("CURSEFORGE_FILE_CDN_URL")
                .unwrap_or_else(|_| "https://mediafilez.forgecdn.net".to_string()),
            modrinth_file_cdn_url: env::var("MODRINTH_FILE_CDN_URL")
                .unwrap_or_else(|_| "https://cdn.modrinth.com".to_string()),
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
            .wrap(Logger::new(
                "%a \"%r\" %s \"%{Referer}i\" \"%{User-Agent}i\" %D ms",
            ))
            .route("/openapi.json", web::get().to(serve_openapi))
    };

    log::info!("🚀 Server starting on http://0.0.0.0:{}", port);

    HttpServer::new(app).bind(&bind_address)?.run().await
}
