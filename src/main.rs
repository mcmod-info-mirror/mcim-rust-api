pub mod db;
pub mod errors;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

use actix_middleware_etag::Etag;
use actix_web::dev::Service;
use actix_web::middleware::{Compress, Logger};
use actix_web::{dev::ServiceRequest, web, App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;
use dotenvy::dotenv;
use prometheus::{IntCounterVec, Opts};
use std::env;

use crate::db::_redis::connect as connect_redis;
use crate::db::database::connect as connect_mongo;
use crate::errors::ApiError;
use crate::routes::config as routes_config;
use crate::utils::app::build_app_state;

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

    let app_state = build_app_state(mongo_client, redis_pool);
    let app_data = web::Data::new(app_state);

    // Prometheus Metrics 初始化
    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .mask_unmatched_patterns("UNKNOWN")
        .build()
        .unwrap();

    // 自定义指标：统计 User-Agent
    let opts = Opts::new(
        "http_requests_user_agent",
        "Number of HTTP requests by User-Agent",
    )
    .namespace("api");

    let user_agent_counter = IntCounterVec::new(opts, &["user_agent"]).unwrap();

    prometheus
        .registry
        .register(Box::new(user_agent_counter.clone()))
        .unwrap();

    // 将 counter 存入 AppData 供中间件使用
    let user_agent_counter_data = web::Data::new(user_agent_counter);

    let app = move || {
        let logger = Logger::new(
            "%{r}a \"%r\" \"%{RoutePattern}xi\" %s \"%{Referer}i\" \"%{User-Agent}i\" %D ms",
        )
        .custom_request_replace("RoutePattern", |req: &ServiceRequest| {
            req.request()
                .match_pattern()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_string())
        });

        App::new()
            .app_data(app_data.clone())
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
            .app_data(user_agent_counter_data.clone())
            .wrap_fn(|req, srv| {
                // 提取 User-Agent
                let user_agent = req
                    .headers()
                    .get("User-Agent")
                    .and_then(|hv| hv.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string();

                // 获取 counter 并增加
                let counter = req.app_data::<web::Data<IntCounterVec>>().unwrap();
                counter.with_label_values(&[&user_agent]).inc();

                srv.call(req)
            })
            .wrap(Etag)
            .wrap(Compress::default())
            .wrap(prometheus.clone())
            .wrap(logger)
            .configure(routes_config)
    };

    log::info!("🚀 Server starting on http://0.0.0.0:{}", port);

    HttpServer::new(app).bind(&bind_address)?.run().await
}
