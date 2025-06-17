use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use super::entities::{Category, File, Fingerprint, Mod};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Pagination {
    pub index: i32,
    pub page_size: i32,
    pub result_count: i32,
    pub total_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FingerprintResult {
    #[serde(rename = "isCacheBuilt")]
    pub is_cache_built: bool,
    #[serde(rename = "exactMatches")]
    pub exact_matches: Vec<Fingerprint>,
    #[serde(rename = "exactFingerprints")]
    pub exact_fingerprints: Vec<i32>,
    #[serde(rename = "installedFingerprints")]
    pub installed_fingerprints: Vec<i32>,
    #[serde(rename = "unmatchedFingerprints")]
    pub unmatched_fingerprints: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchResponse {
    pub data: Vec<Mod>,
    pub pagination: Pagination,
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

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModInfo {
    pub id: i64,
    pub game_id: i64,
    pub name: String,
    pub slug: String,
    pub links: Links,
    pub summary: String,
    pub status: i64,
    pub download_count: i64,
    pub is_featured: bool,
    pub primary_category_id: i64,
    pub categories: Vec<CategoryInfo>,
    pub class_id: i64,
    pub authors: Vec<Author>,
    pub logo: Logo,
    pub screenshots: Vec<Screenshot>,
    pub main_file_id: i64,
    pub latest_files: Vec<LatestFile>,
    pub latest_files_indexes: Vec<LatestFilesIndex>,
    pub date_created: String,
    pub date_modified: String,
    pub date_released: String,
    pub allow_mod_distribution: bool,
    pub game_popularity_rank: i64,
    pub is_available: bool,
    pub thumbs_up_count: i64,
    pub rating: Value,
    #[serde(rename = "sync_at")]
    pub sync_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    pub website_url: String,
    pub wiki_url: String,
    pub issues_url: Value,
    pub source_url: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryInfo {
    pub id: i64,
    pub game_id: i64,
    pub name: String,
    pub slug: String,
    pub url: String,
    pub icon_url: String,
    pub date_modified: String,
    pub is_class: bool,
    pub class_id: i64,
    pub parent_category_id: i64,
    pub display_index: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Logo {
    pub id: i64,
    pub mod_id: i64,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Screenshot {
    pub id: i64,
    pub mod_id: i64,
    pub title: String,
    pub description: String,
    pub thumbnail_url: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LatestFile {
    pub id: i64,
    pub game_id: i64,
    pub mod_id: i64,
    pub is_available: bool,
    pub display_name: String,
    pub file_name: String,
    pub release_type: i64,
    pub file_status: i64,
    pub hashes: Vec<Hash>,
    pub file_date: String,
    pub file_length: i64,
    pub download_count: i64,
    pub file_size_on_disk: i64,
    pub download_url: Value,
    pub game_versions: Vec<String>,
    pub sortable_game_versions: Vec<SortableGameVersion>,
    pub dependencies: Vec<Value>,
    pub expose_as_alternative: Value,
    pub parent_project_file_id: Value,
    pub alternate_file_id: i64,
    pub is_server_pack: bool,
    pub server_pack_file_id: Value,
    pub is_early_access_content: Value,
    pub early_access_end_date: Value,
    pub file_fingerprint: i64,
    pub modules: Vec<Module>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Hash {
    pub value: String,
    pub algo: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SortableGameVersion {
    pub game_version_name: String,
    pub game_version_padded: String,
    pub game_version: String,
    pub game_version_release_date: String,
    pub game_version_type_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Module {
    pub name: String,
    pub fingerprint: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LatestFilesIndex {
    pub game_version: String,
    pub file_id: i64,
    pub filename: String,
    pub release_type: i64,
    pub game_version_type_id: i64,
    pub mod_loader: Value,
}
