use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::models::curseforge::entities as db;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct Category {
    pub id: i32,
    #[serde(rename = "gameId")]
    pub game_id: i32,
    pub name: String,
    pub slug: Option<String>,
    pub url: String,
    #[serde(rename = "iconUrl")]
    pub icon_url: String,
    #[serde(rename = "dateModified")]
    pub date_modified: DateTime<Utc>,
    #[serde(rename = "isClass")]
    pub is_class: Option<bool>,
    #[serde(rename = "classId")]
    pub class_id: Option<i32>,
    #[serde(rename = "parentCategoryId")]
    pub parent_category_id: Option<i32>,
    #[serde(rename = "displayIndex")]
    pub display_index: i32,

    #[serde(default = "Utc::now")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct CategoryInfo {
    pub id: Option<i32>,
    #[serde(rename = "gameId")]
    pub game_id: Option<i32>,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    #[serde(rename = "dateModified")]
    pub date_modified: Option<DateTime<Utc>>,
    #[serde(rename = "isClass")]
    pub is_class: Option<bool>,
    #[serde(rename = "classId")]
    pub class_id: Option<i32>,
    #[serde(rename = "parentCategoryId")]
    pub parent_category_id: Option<i32>,
    #[serde(rename = "displayIndex")]
    pub display_index: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct File {
    pub id: i32,
    #[serde(rename = "gameId")]
    pub game_id: i32,
    #[serde(rename = "modId")]
    pub mod_id: i32,
    #[serde(rename = "isAvailable")]
    pub is_available: Option<bool>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "releaseType")]
    pub release_type: Option<i32>,
    #[serde(rename = "fileStatus")]
    pub file_status: Option<i32>,
    pub hashes: Option<Vec<Hash>>,
    #[serde(rename = "fileDate")]
    pub file_date: Option<DateTime<Utc>>,
    #[serde(rename = "fileLength")]
    pub file_length: Option<i64>,
    #[serde(rename = "downloadCount")]
    pub download_count: Option<i64>,
    #[serde(rename = "fileSizeOnDisk")]
    pub file_size_on_disk: Option<i64>,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Option<Vec<String>>,
    #[serde(rename = "sortableGameVersions")]
    pub sortable_game_versions: Option<Vec<FileSortableGameVersions>>,
    pub dependencies: Option<Vec<FileDependencies>>,
    #[serde(rename = "exposeAsAlternative")]
    pub expose_as_alternative: Option<bool>,
    #[serde(rename = "parentProjectFileId")]
    pub parent_project_file_id: Option<i32>,
    #[serde(rename = "alternateFileId")]
    pub alternate_file_id: Option<i32>,
    #[serde(rename = "isServerPack")]
    pub is_server_pack: Option<bool>,
    #[serde(rename = "serverPackFileId")]
    pub server_pack_file_id: Option<i32>,
    #[serde(rename = "isEarlyAccessContent")]
    pub is_early_access_content: Option<bool>,
    #[serde(rename = "earlyAccessEndDate")]
    pub early_access_end_date: Option<DateTime<Utc>>,
    #[serde(rename = "fileFingerprint")]
    pub file_fingerprint: Option<i64>,
    pub modules: Option<Vec<Module>>,

    #[serde(default = "Utc::now")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct FileInfo {
    pub id: i32,
    #[serde(rename = "gameId")]
    pub game_id: i32,
    #[serde(rename = "modId")]
    pub mod_id: i32,
    #[serde(rename = "isAvailable")]
    pub is_available: Option<bool>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "releaseType")]
    pub release_type: Option<i32>,
    #[serde(rename = "fileStatus")]
    pub file_status: Option<i32>,
    pub hashes: Option<Vec<Hash>>,
    #[serde(rename = "fileDate")]
    pub file_date: Option<DateTime<Utc>>,
    #[serde(rename = "fileLength")]
    pub file_length: Option<i64>,
    #[serde(rename = "downloadCount")]
    pub download_count: Option<i64>,
    #[serde(rename = "fileSizeOnDisk")]
    pub file_size_on_disk: Option<i64>,
    #[serde(rename = "downloadUrl")]
    pub download_url: Option<String>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Option<Vec<String>>,
    #[serde(rename = "sortableGameVersions")]
    pub sortable_game_versions: Option<Vec<FileSortableGameVersions>>,
    pub dependencies: Option<Vec<FileDependencies>>,
    #[serde(rename = "exposeAsAlternative")]
    pub expose_as_alternative: Option<bool>,
    #[serde(rename = "parentProjectFileId")]
    pub parent_project_file_id: Option<i32>,
    #[serde(rename = "alternateFileId")]
    pub alternate_file_id: Option<i32>,
    #[serde(rename = "isServerPack")]
    pub is_server_pack: Option<bool>,
    #[serde(rename = "serverPackFileId")]
    pub server_pack_file_id: Option<i32>,
    #[serde(rename = "isEarlyAccessContent")]
    pub is_early_access_content: Option<bool>,
    #[serde(rename = "earlyAccessEndDate")]
    pub early_access_end_date: Option<DateTime<Utc>>,
    #[serde(rename = "fileFingerprint")]
    pub file_fingerprint: Option<i64>,
    pub modules: Option<Vec<Module>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct Mod {
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

    #[serde(default = "Utc::now")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct Fingerprint {
    pub id: i32,
    pub file: FileInfo,
    #[serde(rename = "latestFiles")]
    pub latest_files: Vec<FileInfo>,

    #[serde(default = "Utc::now")]
    pub sync_at: DateTime<Utc>,
}

// --- Re-usable sub-structs (mirrored from entities) ---

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct FileDependencies {
    #[serde(rename = "modId")]
    pub mod_id: i32,
    #[serde(rename = "relationType")]
    pub relation_type: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct FileSortableGameVersions {
    #[serde(rename = "gameVersionName")]
    pub game_version_name: Option<String>,
    #[serde(rename = "gameVersionPadded")]
    pub game_version_padded: Option<String>,
    #[serde(rename = "gameVersion")]
    pub game_version: Option<String>,
    #[serde(rename = "gameVersionReleaseDate")]
    pub game_version_release_date: Option<DateTime<Utc>>,
    #[serde(rename = "gameVersionTypeId")]
    pub game_version_type_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct Hash {
    pub value: String,
    pub algo: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct Author {
    pub id: i32,
    pub name: String,
    pub url: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct Logo {
    pub id: i32,
    #[serde(rename = "modId")]
    pub mod_id: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct Links {
    #[serde(rename = "websiteUrl")]
    pub website_url: Option<String>,
    #[serde(rename = "wikiUrl")]
    pub wiki_url: Option<String>,
    #[serde(rename = "issuesUrl")]
    pub issues_url: Option<String>,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct ScreenShot {
    pub id: i32,
    #[serde(rename = "modId")]
    pub mod_id: i32,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct Module {
    pub name: Option<String>,
    pub fingerprint: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct FileIndex {
    #[serde(rename = "gameVersion")]
    pub game_version: Option<String>,
    #[serde(rename = "fileId")]
    pub file_id: i32,
    pub filename: Option<String>,
    #[serde(rename = "releaseType")]
    pub release_type: Option<i32>,
    #[serde(rename = "gameVersionTypeId")]
    pub game_version_type_id: Option<i32>,
    #[serde(rename = "modLoader", skip_serializing_if = "Option::is_none")]
    // https://github.com/Meloong-Git/PCL/issues/6656#issuecomment-3121181878
    pub mod_loader: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct FingerprintResult {
    #[serde(rename = "isCacheBuilt")]
    pub is_cache_built: bool,
    #[serde(rename = "exactMatches")]
    pub exact_matches: Vec<Fingerprint>, // Changed from SingleFingerprintResponse
    #[serde(rename = "exactFingerprints")]
    pub exact_fingerprints: Vec<i64>,
    #[serde(rename = "installedFingerprints")]
    pub installed_fingerprints: Vec<i64>,
    #[serde(rename = "unmatchedFingerprints")]
    pub unmatched_fingerprints: Option<Vec<i64>>,
    #[serde(rename = "partialMatches")]
    pub partial_matches: Vec<Fingerprint>,
    #[serde(rename = "partialMatchFingerprints")]
    pub partial_match_fingerprints: HashMap<i64, Vec<i64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct SearchResponse {
    pub data: Vec<Mod>, // Changed from ModInfo
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]

pub struct Pagination {
    pub index: i32,
    #[serde(rename = "pageSize")]
    pub page_size: i32,
    #[serde(rename = "resultCount")]
    pub result_count: i32,
    #[serde(rename = "totalCount")]
    pub total_count: i32,
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

// --- Conversions from DB Entity to Response Model ---

// Macro to reduce boilerplate for simple struct conversions
macro_rules! impl_from_db {
    ($db_type:ty, $resp_type:ty) => {
        impl From<$db_type> for $resp_type {
            fn from(db_model: $db_type) -> Self {
                // This is a bit of a trick to convert between two structs with identical fields.
                // It serializes the source and deserializes into the target.
                // This requires both structs to have compatible serde implementations.
                // A manual field-by-field copy is safer if fields differ.
                serde_json::from_value(serde_json::to_value(db_model).unwrap()).unwrap()
            }
        }
    };
}

impl_from_db!(db::FileDependencies, FileDependencies);
impl_from_db!(db::FileSortableGameVersions, FileSortableGameVersions);
impl_from_db!(db::Hash, Hash);
impl_from_db!(db::Author, Author);
impl_from_db!(db::Logo, Logo);
impl_from_db!(db::Links, Links);
impl_from_db!(db::ScreenShot, ScreenShot);
impl_from_db!(db::Module, Module);
impl_from_db!(db::FileIndex, FileIndex);

impl From<db::Category> for Category {
    fn from(db_model: db::Category) -> Self {
        Self {
            id: db_model.id,
            game_id: db_model.game_id,
            name: db_model.name,
            slug: db_model.slug,
            url: db_model.url,
            icon_url: db_model.icon_url,
            date_modified: db_model.date_modified,
            is_class: db_model.is_class,
            class_id: db_model.class_id,
            parent_category_id: db_model.parent_category_id,
            display_index: db_model.display_index,
            sync_at: db_model.sync_at,
        }
    }
}

impl From<db::CategoryInfo> for CategoryInfo {
    fn from(db_model: db::CategoryInfo) -> Self {
        Self {
            id: db_model.id,
            game_id: db_model.game_id,
            name: db_model.name,
            slug: db_model.slug,
            url: db_model.url,
            icon_url: db_model.icon_url,
            date_modified: db_model.date_modified,
            is_class: db_model.is_class,
            class_id: db_model.class_id,
            parent_category_id: db_model.parent_category_id,
            display_index: db_model.display_index,
        }
    }
}

impl From<db::File> for File {
    fn from(db_model: db::File) -> Self {
        Self {
            id: db_model.id,
            game_id: db_model.game_id,
            mod_id: db_model.mod_id,
            is_available: db_model.is_available,
            display_name: db_model.display_name,
            file_name: db_model.file_name,
            release_type: db_model.release_type,
            file_status: db_model.file_status,
            hashes: db_model
                .hashes
                .map(|v| v.into_iter().map(Into::into).collect()),
            file_date: db_model.file_date,
            file_length: db_model.file_length,
            download_count: db_model.download_count,
            file_size_on_disk: db_model.file_size_on_disk,
            download_url: db_model.download_url,
            game_versions: db_model.game_versions,
            sortable_game_versions: db_model
                .sortable_game_versions
                .map(|v| v.into_iter().map(Into::into).collect()),
            dependencies: db_model
                .dependencies
                .map(|v| v.into_iter().map(Into::into).collect()),
            expose_as_alternative: db_model.expose_as_alternative,
            parent_project_file_id: db_model.parent_project_file_id,
            alternate_file_id: db_model.alternate_file_id,
            is_server_pack: db_model.is_server_pack,
            server_pack_file_id: db_model.server_pack_file_id,
            is_early_access_content: db_model.is_early_access_content,
            early_access_end_date: db_model.early_access_end_date,
            file_fingerprint: db_model.file_fingerprint,
            modules: db_model
                .modules
                .map(|v| v.into_iter().map(Into::into).collect()),
            sync_at: db_model.sync_at,
        }
    }
}

impl From<db::FileInfo> for FileInfo {
    fn from(db_model: db::FileInfo) -> Self {
        Self {
            id: db_model.id,
            game_id: db_model.game_id,
            mod_id: db_model.mod_id,
            is_available: db_model.is_available,
            display_name: db_model.display_name,
            file_name: db_model.file_name,
            release_type: db_model.release_type,
            file_status: db_model.file_status,
            hashes: db_model
                .hashes
                .map(|v| v.into_iter().map(Into::into).collect()),
            file_date: db_model.file_date,
            file_length: db_model.file_length,
            download_count: db_model.download_count,
            file_size_on_disk: db_model.file_size_on_disk,
            download_url: db_model.download_url,
            game_versions: db_model.game_versions,
            sortable_game_versions: db_model
                .sortable_game_versions
                .map(|v| v.into_iter().map(Into::into).collect()),
            dependencies: db_model
                .dependencies
                .map(|v| v.into_iter().map(Into::into).collect()),
            expose_as_alternative: db_model.expose_as_alternative,
            parent_project_file_id: db_model.parent_project_file_id,
            alternate_file_id: db_model.alternate_file_id,
            is_server_pack: db_model.is_server_pack,
            server_pack_file_id: db_model.server_pack_file_id,
            is_early_access_content: db_model.is_early_access_content,
            early_access_end_date: db_model.early_access_end_date,
            file_fingerprint: db_model.file_fingerprint,
            modules: db_model
                .modules
                .map(|v| v.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<db::Mod> for Mod {
    fn from(db_model: db::Mod) -> Self {
        Self {
            id: db_model.id,
            game_id: db_model.game_id,
            name: db_model.name,
            slug: db_model.slug,
            links: db_model.links.map(Into::into),
            summary: db_model.summary,
            status: db_model.status,
            download_count: db_model.download_count,
            is_featured: db_model.is_featured,
            primary_category_id: db_model.primary_category_id,
            categories: db_model
                .categories
                .map(|v| v.into_iter().map(Into::into).collect()),
            class_id: db_model.class_id,
            authors: db_model
                .authors
                .map(|v| v.into_iter().map(Into::into).collect()),
            logo: db_model.logo.map(Into::into),
            screenshots: db_model
                .screenshots
                .map(|v| v.into_iter().map(Into::into).collect()),
            main_file_id: db_model.main_file_id,
            latest_files: db_model
                .latest_files
                .map(|v| v.into_iter().map(Into::into).collect()),
            latest_files_indexes: db_model
                .latest_files_indexes
                .map(|v| v.into_iter().map(Into::into).collect()),
            date_created: db_model.date_created,
            date_modified: db_model.date_modified,
            date_released: db_model.date_released,
            allow_mod_distribution: db_model.allow_mod_distribution,
            game_popularity_rank: db_model.game_popularity_rank,
            is_available: db_model.is_available,
            thumbs_up_count: db_model.thumbs_up_count,
            rating: db_model.rating,
            sync_at: db_model.sync_at,
        }
    }
}

impl From<db::Fingerprint> for Fingerprint {
    fn from(db_model: db::Fingerprint) -> Self {
        Self {
            id: db_model.file.mod_id, // Use mod_id as id
            file: db_model.file.into(),
            latest_files: db_model.latest_files.into_iter().map(Into::into).collect(),
            sync_at: db_model.sync_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use std::env;

    fn get_base_url() -> String {
        dotenv().ok();
        env::var("CURSEFORGE_API_ENDPOINT")
            .unwrap_or_else(|_| "https://api.curseforge.com/v1".to_string())
    }

    fn get_x_api_key() -> String {
        dotenv().ok();
        env::var("CURSEFORGE_API_KEY").expect("CURSEFORGE_API_KEY not set")
    }

    #[tokio::test]
    async fn test_mod_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let api_key = get_x_api_key();

        let response = client
            .get(&format!("{}/mods/238222", base_url)) // JEI mod
            .header("x-api-key", api_key)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let mod_response: ModResponse = response
            .json()
            .await
            .expect("Failed to deserialize ModResponse");

        assert_eq!(mod_response.data.id, 238222);
    }

    #[tokio::test]
    async fn test_search_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let api_key = get_x_api_key();

        let response = client
            .get(&format!(
                "{}/mods/search?gameId=432&classId=6&pageSize=2",
                base_url
            ))
            .header("x-api-key", api_key)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let search_response: SearchResponse = response
            .json()
            .await
            .expect("Failed to deserialize SearchResponse");

        assert!(!search_response.data.is_empty());
        assert_eq!(search_response.pagination.page_size, 2);
    }

    #[tokio::test]
    async fn test_mods_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let api_key = get_x_api_key();

        let payload = serde_json::json!({ "modIds": [946010, 594678] });

        let response = client
            .post(&format!("{}/mods", base_url))
            .header("x-api-key", api_key)
            .json(&payload)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let mods_response: ModsResponse = response
            .json()
            .await
            .expect("Failed to deserialize ModsResponse");

        assert_eq!(mods_response.data.len(), 2);
    }

    #[tokio::test]
    async fn test_file_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let api_key = get_x_api_key();
        let mod_id = 594678;
        let file_id = 3913840;

        let response = client
            .get(&format!("{}/mods/{}/files/{}", base_url, mod_id, file_id))
            .header("x-api-key", api_key)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let file_response: FileResponse = response
            .json()
            .await
            .expect("Failed to deserialize FileResponse");

        assert_eq!(file_response.data.id, file_id);
        assert_eq!(file_response.data.mod_id, mod_id);
    }

    #[tokio::test]
    async fn test_files_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let api_key = get_x_api_key();

        let payload = serde_json::json!({ "fileIds": [3913840, 5976953] }); // JEI files

        let response = client
            .post(&format!("{}/mods/files", base_url))
            .header("x-api-key", api_key)
            .json(&payload)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let files_response: FilesResponse = response
            .json()
            .await
            .expect("Failed to deserialize FilesResponse");

        assert_eq!(files_response.data.len(), 2);
    }

    #[tokio::test]
    async fn test_mod_files_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let api_key = get_x_api_key();
        let mod_id = 238222;
        let page_size = 5;

        let response = client
            .get(&format!(
                "{}/mods/{}/files?pageSize={}",
                base_url, mod_id, page_size
            ))
            .header("x-api-key", api_key)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let mod_files_response: ModFilesResponse = response
            .json()
            .await
            .expect("Failed to deserialize ModFilesResponse");

        assert_eq!(mod_files_response.data.len(), page_size);
        assert!(mod_files_response.data.iter().all(|f| f.mod_id == mod_id));
    }

    #[tokio::test]
    async fn test_download_url_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let api_key = get_x_api_key();
        let mod_id = 594678;
        let file_id = 3913840;

        let response = client
            .get(&format!(
                "{}/mods/{}/files/{}/download-url",
                base_url, mod_id, file_id
            ))
            .header("x-api-key", api_key)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let download_url_response: DownloadUrlResponse = response
            .json()
            .await
            .expect("Failed to deserialize DownloadUrlResponse");

        assert!(download_url_response
            .data
            .starts_with("https://edge.forgecdn.net/"));
    }

    #[tokio::test]
    async fn test_fingerprint_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let api_key = get_x_api_key();

        let payload = serde_json::json!({ "fingerprints": [2070800629, 1904165976, 9999] });

        let response = client
            .post(&format!("{}/fingerprints", base_url))
            .header("x-api-key", api_key)
            .json(&payload)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let fingerprint_response_json: FingerprintResponse = response
            .json()
            .await
            .expect("Failed to deserialize FingerprintResponse");

        assert!(fingerprint_response_json.data.is_cache_built);
        assert_eq!(
            fingerprint_response_json.data.exact_matches.len() == 2,
            true
        );
        assert!(fingerprint_response_json
            .data
            .exact_fingerprints
            .contains(&2070800629));
        assert!(fingerprint_response_json
            .data
            .exact_fingerprints
            .contains(&1904165976));
        // 这里即便有传 9999，unmatched_fingerprints 也一样是 Null，更不是列表
        // MCIM 会返回未匹配的指纹列表
        assert!(fingerprint_response_json
            .data
            .unmatched_fingerprints
            .is_none());
        assert_eq!(
            fingerprint_response_json.data.installed_fingerprints.len() == 3,
            true
        );
    }

    #[tokio::test]
    async fn test_categories_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let api_key = get_x_api_key();

        let response = client
            .get(&format!("{}/categories?gameId=432", base_url)) // Minecraft
            .header("x-api-key", api_key)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let categories_response: CategoriesResponse = response
            .json()
            .await
            .expect("Failed to deserialize CategoriesResponse");

        assert!(!categories_response.data.is_empty());
        assert!(categories_response.data.iter().all(|c| c.game_id == 432));
    }
}
