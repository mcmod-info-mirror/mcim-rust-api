//! CurseForge 数据库模型。
//!
//! - `sqlx::FromRow`：读取 PostgreSQL 主表 / 子表行；`#[sqlx(skip)]` 字段由子表另行填充。
//! - `serde::Deserialize`：字段命名与 CurseForge API / 原 MongoDB 文档一致，供迁移和同步写入使用。

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::FromRow;

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Category {
    #[serde(alias = "_id")]
    pub id: i32,
    #[serde(rename = "gameId")]
    pub game_id: i32,
    pub name: String,
    pub slug: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
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

    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
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

/// `curseforge_files` 集合中的文件文档：文件内容 + 同步状态
#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct File {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub info: FileInfo,

    pub sync_at: DateTime<Utc>,
    pub found: Option<bool>,
    pub file_cdn_cached: Option<bool>,
    pub need_to_cache: Option<bool>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
}

/// 文件内容（`curseforge_file_records`），亦用于 Mod.latestFiles
#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct FileInfo {
    #[serde(alias = "_id")]
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
    #[sqlx(skip)]
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
    #[sqlx(skip)]
    pub game_versions: Option<Vec<String>>,
    #[serde(rename = "sortableGameVersions")]
    #[sqlx(skip)]
    pub sortable_game_versions: Option<Vec<FileSortableGameVersions>>,
    #[sqlx(skip)]
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
    #[sqlx(skip)]
    pub modules: Option<Vec<Module>>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Mod {
    #[serde(alias = "_id")]
    pub id: i32,
    #[serde(rename = "gameId")]
    pub game_id: Option<i32>,
    pub name: Option<String>,
    pub slug: String,
    #[sqlx(skip)]
    pub links: Option<Links>,
    pub summary: Option<String>,
    pub status: Option<i32>,
    #[serde(rename = "downloadCount")]
    pub download_count: Option<i64>,
    #[serde(rename = "isFeatured")]
    pub is_featured: Option<bool>,
    #[serde(rename = "primaryCategoryId")]
    pub primary_category_id: Option<i32>,
    #[sqlx(skip)]
    pub categories: Option<Vec<CategoryInfo>>,
    #[serde(rename = "classId")]
    pub class_id: Option<i32>,
    #[sqlx(skip)]
    pub authors: Option<Vec<Author>>,
    #[sqlx(skip)]
    pub logo: Option<Logo>,
    #[sqlx(skip)]
    pub screenshots: Option<Vec<ScreenShot>>,
    #[serde(rename = "mainFileId")]
    pub main_file_id: Option<i32>,
    #[serde(rename = "latestFiles")]
    #[sqlx(skip)]
    pub latest_files: Option<Vec<FileInfo>>,
    #[serde(rename = "latestFilesIndexes")]
    #[sqlx(skip)]
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
    pub translated_summary: Option<String>,

    pub sync_at: DateTime<Utc>,
    pub checked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct FileDependencies {
    #[serde(rename = "modId")]
    pub mod_id: i32,
    #[serde(rename = "relationType")]
    pub relation_type: Option<i32>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
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

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Hash {
    pub value: String,
    pub algo: i32,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Author {
    pub id: i32,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
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

#[derive(Debug, Deserialize, Clone, FromRow)]
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

#[derive(Debug, Deserialize, Clone, FromRow)]
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

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Module {
    pub name: Option<String>,
    pub fingerprint: Option<i64>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
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
    #[serde(rename = "modLoader")]
    pub mod_loader: Option<i32>,
}
