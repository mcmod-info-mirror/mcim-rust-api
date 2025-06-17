use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema};
use std::collections::HashMap;

use bson::serde_helpers::chrono_datetime_as_bson_datetime;
use crate::models::modrinth::entities::*;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct MutilFilesResponse {
    #[serde(flatten)]
    pub entries: HashMap<String, Version>,
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
    #[serde(rename = "project_id")]
    pub project_id: String,
    #[serde(rename = "project_type")]
    pub project_type: String,
    pub slug: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    #[serde(rename = "display_categories")]
    pub display_categories: Option<Vec<String>>,
    pub versions: Vec<String>,
    pub downloads: i64,
    pub follows: i32,
    #[serde(rename = "icon_url")]
    pub icon_url: String,
    #[serde(rename = "date_created", with = "chrono_datetime_as_bson_datetime")]
    pub date_created: DateTime<Utc>,
    #[serde(rename = "date_modified", with = "chrono_datetime_as_bson_datetime")]
    pub date_modified: DateTime<Utc>,
    #[serde(rename = "latest_version")]
    pub latest_version: Option<String>,
    pub license: String,
    #[serde(rename = "client_side")]
    pub client_side: String,
    #[serde(rename = "server_side")]
    pub server_side: String,
    pub gallery: Option<Vec<String>>,
    #[serde(rename = "featured_gallery")]
    pub featured_gallery: Option<String>,
    pub color: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub offset: i32,
    pub limit: i32,
    #[serde(rename = "total_hits")]
    pub total_hits: i32,
}