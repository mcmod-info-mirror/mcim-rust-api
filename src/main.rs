pub mod config;
pub mod errors;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

use actix_web::{web, App, HttpServer, dev::ServiceRequest, middleware::Logger};
use dotenvy::dotenv;
use std::env;

use crate::config::_redis::connect as connect_redis;
use crate::config::database::connect as connect_mongo;
use crate::errors::ApiError;
use crate::routes::config as routes_config;

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



    let app = move || {
        let logger =
        Logger::new("%a \"%r\" \"%{RoutePattern}xi\" %s \"%{Referer}i\" \"%{User-Agent}i\" %D ms")
            .custom_request_replace("RoutePattern", |req: &ServiceRequest| {
                req.request()
                    .match_pattern()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "-".to_string())
            });

        App::new()
            .app_data(web::Data::new(mcim_rust_api::build_app_state(
                mongo_client.clone(),
                redis_pool.clone(),
            )))
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
            .wrap(logger)
            .configure(routes_config)
    };

    log::info!("🚀 Server starting on http://0.0.0.0:{}", port);

    HttpServer::new(app).bind(&bind_address)?.run().await
}
