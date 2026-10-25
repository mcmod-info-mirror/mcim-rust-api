use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct StatisticsQuery {
    pub curseforge: Option<bool>,
    pub modrinth: Option<bool>,
    pub translate: Option<bool>,
}

#[derive(Deserialize)]
pub struct FileStatisticsQuery {
    pub date: Option<NaiveDate>,
}
