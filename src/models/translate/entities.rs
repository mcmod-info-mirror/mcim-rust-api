use serde::{Serialize, Deserialize};
use chrono::DateTime;
use chrono::Utc;

use bson::serde_helpers::chrono_datetime_as_bson_datetime;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthTranslation {
    #[serde(alias = "_id")]
    pub project_id: String,
    pub translated: String,
    pub original: String,

    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub translated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurseForgeTranslation {
    #[serde(alias = "_id")]
    pub modid: i32,
    pub translated: String,
    pub original: String,

    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub translated_at: DateTime<Utc>,
}