use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::deserialize_bson_datetime_flexible;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(rename = "date_created", deserialize_with = "deserialize_bson_datetime_flexible")]
    pub date_created: DateTime<Utc>,
    #[serde(rename = "date_modified", deserialize_with = "deserialize_bson_datetime_flexible")]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub offset: i32,
    pub limit: i32,
    #[serde(rename = "total_hits")]
    pub total_hits: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoryInfo {
    pub icon: String,
    pub name: String,
    #[serde(rename = "project_type")]
    pub project_type: String,
    pub header: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoaderInfo {
    pub icon: String,
    pub name: String,
    #[serde(rename = "supported_project_types")]
    pub supported_project_types: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameVersionInfo {
    pub version: String,
    #[serde(rename = "version_type")]
    pub version_type: String,
    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub date: DateTime<Utc>,
    pub major: bool,
}