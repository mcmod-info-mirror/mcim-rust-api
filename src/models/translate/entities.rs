use serde::{Serialize, Deserialize};
use chrono::DateTime;
use chrono::Utc;

use bson::serde_helpers::chrono_datetime_as_bson_datetime;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthTranslation {
    #[serde(rename = "_id")]
    pub project_id: String,
    pub translated: String,
    pub original: String,

    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub translated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurseForgeTranslation {
    #[serde(rename = "_id")]
    pub modid: i32,
    pub translated: String,
    pub original: String,

    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub translated_at: DateTime<Utc>,
}