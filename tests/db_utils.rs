use mongodb::{options::ClientOptions, Client};

/// 初始化 MongoDB 客户端（每个测试都可以调用，不需要全局）
pub async fn init_test_db() -> Client {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB connection string");

    Client::with_options(client_options).expect("Failed to initialize MongoDB client")
}
