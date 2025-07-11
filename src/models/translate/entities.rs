use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct ModrinthTranslation {
    pub project_id: String,
    pub translated: String,
    pub original: String,
    pub translated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct CurseForgeTranslation {
    #[serde(rename = "modId")]
    pub mod_id: i32,
    pub translated: String,
    pub original: String,
    pub translated_at: DateTime<Utc>,
}
