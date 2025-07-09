// use mongodb::{options::ClientOptions, Client};
// use std::env;

// pub async fn connect() -> Result<Client, Box<dyn std::error::Error>> {
//     let uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

//     let client_options = ClientOptions::parse(&uri).await?;
//     let client = Client::with_options(client_options)?;

//     // 测试连接
//     client
//         .database("admin")
//         .run_command(mongodb::bson::doc! {"ping": 1}, None)
//         .await?;

//     log::info!("Connected to MongoDB successfully!");
//     Ok(client)
// }

// pub fn get_database_name() -> String {
//     env::var("DATABASE_NAME").unwrap_or_else(|_| "mcim_backend".to_string())
// }

use std::{env, sync::Arc};

use actix_web::rt::spawn;
use tokio_postgres::{connect as async_postgres_connect, Client, NoTls};

pub async fn connect() -> Result<Arc<Client>, Box<dyn std::error::Error>> {
    let mut uri = env::var("POSTGRES_URI").unwrap_or_else(|_| "host=localhost user=postgres".to_string())
        .to_string();

    uri.push_str(get_database_name().as_str());

    let (client, sock) = async_postgres_connect(&uri, NoTls).await.unwrap();

    // 为了保持存活 Socket
    spawn(async move {
        if let Err(err) = sock.await {
            log::error!("Postgres socket error: {}", err);
        }
    });

    Ok(Arc::new(client))
}

pub fn get_database_name() -> String {
    env::var("DATABASE_NAME").unwrap_or_else(|_| "mcim_backend".to_string())
}