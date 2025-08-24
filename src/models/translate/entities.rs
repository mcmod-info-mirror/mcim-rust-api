use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use bson::serde_helpers::chrono_datetime_as_bson_datetime;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthTranslation {
    #[serde(alias = "_id")]
    pub project_id: String,
    pub translated: Option<String>,
    pub original: Option<String>,

    pub need_to_update: bool,

    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub translated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurseForgeTranslation {
    #[serde(rename = "modId", alias = "_id")]
    pub mod_id: i32,
    pub translated: Option<String>,
    pub original: Option<String>,

    pub need_to_update: bool,

    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub translated_at: DateTime<Utc>,
}
