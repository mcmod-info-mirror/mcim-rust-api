use actix_web::{web, App};

pub mod db_utils;

use db_utils::init_test_db;
use mcim_rust_api::routes::config;

pub async fn create_test_app() -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .app_data(web::Data::new(init_test_db().await))
        .configure(|cfg| {
            config(cfg);
        })
}
