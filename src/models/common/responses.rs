use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct StatisticsResponse {
    pub curseforge: Option<HashMap<String, u64>>,
    pub modrinth: Option<HashMap<String, u64>>,
    pub translate: Option<HashMap<String, u64>>,
}
