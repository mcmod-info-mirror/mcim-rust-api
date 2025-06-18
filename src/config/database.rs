use mongodb::{Client, options::ClientOptions};
use std::env;

pub async fn connect() -> Result<Client, Box<dyn std::error::Error>> {
    let uri = env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    
    let client_options = ClientOptions::parse(&uri).await?;
    let client = Client::with_options(client_options)?;
    
    // 测试连接
    client
        .database("admin")
        .run_command(mongodb::bson::doc! {"ping": 1}, None)
        .await?;
    
    log::info!("Connected to MongoDB successfully!");
    Ok(client)
}

pub fn get_database_name() -> String {
    env::var("DATABASE_NAME").unwrap_or_else(|_| "mcim_backend".to_string())
}