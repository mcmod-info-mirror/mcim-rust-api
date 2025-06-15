pub mod config;
pub mod errors;
pub mod models;
pub mod routes;
pub mod services;

use actix_web::{web, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;
use std::env;

use crate::config::database::connect;
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
    // 配置MongoDB连接
    let client = connect().await.expect("Failed to connect to MongoDB");

    // 获取服务器端口，默认为8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("0.0.0.0:{}", port);

    let app = move || {
        let app_data = web::Data::new(AppState { db: client.clone() });

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
            .service(
                SwaggerUi::new("/docs/{_:.*}").url("/openapi.json", OpenApiDoc::openapi()),
            )
    };

    HttpServer::new(app).bind(&bind_address)?.run().await
}
