use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_with::serde_as;

use bson::serde_helpers::datetime::FromChrono04DateTime;

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct ModrinthTranslation {
    #[serde(alias = "_id")]
    pub project_id: String,
    pub translated: Option<String>,
    pub original: Option<String>,

    pub need_to_update: bool,

    #[serde_as(as = "Option<FromChrono04DateTime>")]
    pub translated_at: Option<DateTime<Utc>>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct CurseForgeTranslation {
    #[serde(rename = "modId", alias = "_id")]
    pub mod_id: i32,
    pub translated: Option<String>,
    pub original: Option<String>,

    pub need_to_update: bool,

    #[serde_as(as = "Option<FromChrono04DateTime>")]
    pub translated_at: Option<DateTime<Utc>>,
}
