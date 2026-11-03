pub mod db;
pub mod errors;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

pub mod test_utils {
    use crate::db::database::connect;
    use crate::routes::config;
    use crate::utils::app::build_app_state;
    use actix_web::{App, web};
    use dotenvy::dotenv;
    use redis::aio::MultiplexedConnection;
    use sqlx::PgPool;
    use std::sync::Arc;

    /// 连接 `DATABASE_URL` 指向的 PostgreSQL 并应用表结构迁移
    pub async fn init_test_db() -> PgPool {
        connect().await.expect("Failed to connect to PostgreSQL")
    }

    pub async fn init_test_redis() -> Arc<MultiplexedConnection> {
        let client = redis::Client::open(crate::db::_redis::get_redis_url())
            .expect("Failed to create Redis client");

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

        let db_pool = init_test_db().await;
        let redis_client = init_test_redis().await;

        App::new()
            .app_data(web::Data::new(build_app_state(db_pool, redis_client)))
            .configure(config)
    }
}
