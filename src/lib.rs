pub mod config;
pub mod models;
pub mod routes;
pub mod services;

pub mod test_utils {
    use actix_web::{web, App};
    use mongodb::{options::ClientOptions, Client};
    use crate::routes::config;
    use crate::config::AppState;

    pub async fn init_test_db() -> Client {
        let client_options = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .expect("Failed to parse MongoDB connection string");

        Client::with_options(client_options).expect("Failed to initialize MongoDB client")
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
        let client = init_test_db().await;

        let app_state = AppState {
            db: client, // 直接使用 client
        };

        App::new()
                .app_data(web::Data::new(app_state)) // 传递 AppState
                .configure(|cfg| {
                    config(cfg);
                })
    }
}