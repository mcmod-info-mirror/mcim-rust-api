use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::FromRow;

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct ModrinthTranslation {
    #[serde(alias = "_id")]
    pub project_id: String,
    pub translated: Option<String>,
    pub original: Option<String>,

    pub need_to_update: bool,

    pub translated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct CurseForgeTranslation {
    #[serde(rename = "modId", alias = "_id")]
    pub mod_id: i32,
    pub translated: Option<String>,
    pub original: Option<String>,

    pub need_to_update: bool,

    pub translated_at: Option<DateTime<Utc>>,
}
