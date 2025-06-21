use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema};
use std::collections::HashMap;

use crate::models::modrinth::entities::*;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct MutilFilesResponse {
    #[serde(flatten)]
    pub entries: Option<HashMap<String, Version>>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CategoriesResponse {
    #[serde(flatten)]
    pub categories: Vec<Category>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct LoadersResponse {
    #[serde(flatten)]
    pub loaders: Vec<Loader>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GameVersionsResponse {
    #[serde(flatten)]
    pub game_versions: Vec<GameVersion>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchHit {
    pub project_id: String,
    pub project_type: String,
    pub slug: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub display_categories: Option<Vec<String>>,
    pub versions: Vec<String>,
    pub downloads: i64,
    pub follows: i32,
    pub icon_url: String,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub latest_version: Option<String>,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
    pub gallery: Option<Vec<String>>,
    pub featured_gallery: Option<String>,
    pub color: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub offset: i32,
    pub limit: i32,
    pub total_hits: i32,
}