use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema};
use std::collections::HashMap;

use crate::models::deserialize_bson_datetime_flexible;
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

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub offset: i32,
    pub limit: i32,
    #[serde(rename = "total_hits")]
    pub total_hits: i32,
}


// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct CategoryInfo {
//     pub icon: String,
//     pub name: String,
//     #[serde(rename = "project_type")]
//     pub project_type: String,
//     pub header: String,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct LoaderInfo {
//     pub icon: String,
//     pub name: String,
//     #[serde(rename = "supported_project_types")]
//     pub supported_project_types: Vec<String>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct GameVersionInfo {
//     pub version: String,
//     #[serde(rename = "version_type")]
//     pub version_type: String,
//     #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
//     pub date: DateTime<Utc>,
//     pub major: bool,
// }


// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct DonationUrl {
//     pub id: Option<String>,
//     pub platform: Option<String>,
//     pub url: Option<String>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct LicenseInfo {
//     pub id: Option<String>,
//     pub name: Option<String>,
//     pub url: Option<String>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct GalleryItem {
//     pub url: String,
//     pub featured: bool,
//     pub title: Option<String>,
//     pub description: Option<String>,
//     #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
//     pub created: DateTime<Utc>,
//     pub ordering: Option<i32>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct Dependencies {
//     #[serde(rename = "versionId")]
//     pub version_id: Option<String>,
//     #[serde(rename = "projectId")]
//     pub project_id: Option<String>,
//     #[serde(rename = "fileName")]
//     pub file_name: Option<String>,
//     #[serde(rename = "dependencyType")]
//     pub dependency_type: String,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct Hashes {
//     pub sha512: String,
//     pub sha1: String,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct File {
//     #[serde(rename = "_id")]
//     pub hashes: Hashes,
//     pub url: String,
//     pub filename: String,
//     pub primary: bool,
//     pub size: i64,
//     #[serde(rename = "fileType")]
//     pub file_type: Option<String>,
//     #[serde(rename = "versionId")]
//     pub version_id: String,
//     #[serde(rename = "projectId")]
//     pub project_id: String,
//     #[serde(rename = "fileCdnCached")]
//     pub file_cdn_cached: Option<bool>,

//     #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
//     pub sync_at: DateTime<Utc>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct FileInfo {
//     pub hashes: Hashes,
//     pub url: String,
//     pub filename: String,
//     pub primary: bool,
//     pub size: i64,
//     #[serde(rename = "fileType")]
//     pub file_type: Option<String>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct Version {
//     #[serde(rename = "_id")]
//     pub id: String,
//     #[serde(rename = "projectId")]
//     pub project_id: String,
//     pub name: Option<String>,
//     #[serde(rename = "versionNumber")]
//     pub version_number: Option<String>,
//     pub changelog: Option<String>,
//     pub dependencies: Option<Vec<Dependencies>>,
//     #[serde(rename = "gameVersions")]
//     pub game_versions: Option<Vec<String>>,
//     #[serde(rename = "versionType")]
//     pub version_type: Option<String>,
//     pub loaders: Option<Vec<String>>,
//     pub featured: Option<bool>,
//     pub status: Option<String>,
//     #[serde(rename = "requestedStatus")]
//     pub requested_status: Option<String>,
//     #[serde(rename = "authorId")]
//     pub author_id: String,
//     #[serde(rename = "datePublished", deserialize_with = "deserialize_bson_datetime_flexible")]
//     pub date_published: DateTime<Utc>,
//     pub downloads: i64,
//     #[serde(rename = "changelogUrl")]
//     pub changelog_url: Option<String>,
//     pub files: Vec<FileInfo>,

//     #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
//     pub sync_at: DateTime<Utc>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct Category {
//     #[serde(rename = "_id")]
//     pub icon: String,
//     pub name: String,
//     #[serde(rename = "projectType")]
//     pub project_type: String,
//     pub header: String,

//     #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
//     pub sync_at: DateTime<Utc>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct Loader {
//     #[serde(rename = "_id")]
//     pub icon: String,
//     pub name: String,
//     #[serde(rename = "supportedProjectTypes")]
//     pub supported_project_types: Vec<String>,

//     #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
//     pub sync_at: DateTime<Utc>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct GameVersion {
//     #[serde(rename = "_id")]
//     pub version: String,
//     #[serde(rename = "versionType")]
//     pub version_type: String,
//     #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
//     pub date: DateTime<Utc>,
//     pub major: bool,

//     #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
//     pub sync_at: DateTime<Utc>,
// }