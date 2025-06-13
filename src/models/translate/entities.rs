use serde::{Serialize, Deserialize};
use chrono::DateTime;
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthTranslation {
    #[serde(rename = "_id")]
    pub project_id: String,
    pub translated: String,
    pub original: String,
    pub translated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurseForgeTranslation {
    #[serde(rename = "_id")]
    pub modid: i32,
    pub translated: String,
    pub original: String,
    pub translated_at: DateTime<Utc>,
}