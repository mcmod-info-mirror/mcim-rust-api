use serde::Serialize;
use utoipa::{ToSchema};

#[derive(Serialize, ToSchema)]
pub struct StatisticsResponse {
    pub curseforge: Option<i64>,
    pub modrinth: Option<i64>,
    pub translate: Option<i64>,
}