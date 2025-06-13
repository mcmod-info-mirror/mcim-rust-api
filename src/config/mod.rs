pub mod database;

use mongodb::Client;

#[derive(Clone)]
pub struct AppState {
    pub db: Client,
}