use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::curseforge::entities::{Author, FileIndex, Links, Logo, ScreenShot};

use crate::models::curseforge::entities::{Category, CategoryInfo, File, FileInfo, Mod};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FingerprintResult {
    #[serde(rename = "isCacheBuilt")]
    pub is_cache_built: bool,
    #[serde(rename = "exactMatches")]
    pub exact_matches: Vec<SingleFingerprintResponse>,
    #[serde(rename = "exactFingerprints")]
    pub exact_fingerprints: Vec<i64>,
    #[serde(rename = "installedFingerprints")]
    pub installed_fingerprints: Vec<i64>,
    #[serde(rename = "unmatchedFingerprints")]
    pub unmatched_fingerprints: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SingleFingerprintResponse {
    pub id: i32,
    pub file: FileInfo,
    #[serde(rename = "latestFiles")]
    pub latest_files: Vec<FileInfo>,

    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchResponse {
    pub data: Vec<ModInfo>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Pagination {
    pub index: i32,
    pub page_size: i32,
    pub result_count: i32,
    pub total_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModInfo {
    pub id: i32,
    #[serde(rename = "gameId")]
    pub game_id: Option<i32>,
    pub name: Option<String>,
    pub slug: String,
    pub links: Option<Links>,
    pub summary: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "downloadCount")]
    pub download_count: Option<i64>,
    #[serde(rename = "isFeatured")]
    pub is_featured: Option<bool>,
    #[serde(rename = "primaryCategoryId")]
    pub primary_category_id: Option<i32>,
    pub categories: Option<Vec<CategoryInfo>>,
    #[serde(rename = "classId")]
    pub class_id: Option<i32>,
    pub authors: Option<Vec<Author>>,
    pub logo: Option<Logo>,
    pub screenshots: Option<Vec<ScreenShot>>,
    #[serde(rename = "mainFileId")]
    pub main_file_id: Option<i32>,
    #[serde(rename = "latestFiles")]
    pub latest_files: Option<Vec<FileInfo>>,
    #[serde(rename = "latestFilesIndexes")]
    pub latest_files_indexes: Option<Vec<FileIndex>>,
    #[serde(rename = "dateCreated")]
    pub date_created: Option<DateTime<Utc>>,
    #[serde(rename = "dateModified")]
    pub date_modified: Option<DateTime<Utc>>,
    #[serde(rename = "dateReleased")]
    pub date_released: Option<DateTime<Utc>>,
    #[serde(rename = "allowModDistribution")]
    pub allow_mod_distribution: Option<bool>,
    #[serde(rename = "gamePopularityRank")]
    pub game_popularity_rank: Option<i32>,
    #[serde(rename = "isAvailable")]
    pub is_available: Option<bool>,
    #[serde(rename = "thumbsUpCount")]
    pub thumbs_up_count: Option<i32>,
    pub rating: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DownloadUrlResponse {
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModResponse {
    pub data: Mod,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModsResponse {
    pub data: Vec<Mod>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModFilesResponse {
    pub data: Vec<File>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FileResponse {
    pub data: File,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FilesResponse {
    pub data: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FingerprintResponse {
    pub data: FingerprintResult,
}


#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CategoriesResponse {
    pub data: Vec<Category>,
}