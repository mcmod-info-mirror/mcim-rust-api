pub mod config;
pub mod models;
pub mod routes;
pub mod services;

use actix_web::{web, App, HttpServer};
use std::env;

use crate::config::database::connect;
use crate::config::AppState;
use crate::routes::config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 配置MongoDB连接
    let client = connect().await.expect("Failed to connect to MongoDB");

    // 获取服务器端口，默认为8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    // 启动 Actix Web 服务器
    let _ = HttpServer::new(move || {
        let app_data = web::Data::new(AppState { db: client.clone() });

        App::new().app_data(app_data).configure(config)
    })
    .bind(&bind_address)
    .expect("Failed to bind server")
    .run()
    .await;
    Ok(())
}
