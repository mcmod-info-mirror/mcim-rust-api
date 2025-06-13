use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct StatisticsQuery {
    pub curseforge: Option<bool>,
    pub modrinth: Option<bool>,
    pub translate: Option<bool>,
}