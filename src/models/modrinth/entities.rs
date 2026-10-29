//! Modrinth 数据库模型。
//!
//! - `sqlx::FromRow`：读取 PostgreSQL 主表 / 子表行；`#[sqlx(skip)]` 字段由子表另行填充。
//! - `serde::Deserialize`：字段命名与 Modrinth API / 原 MongoDB 文档一致，供迁移和同步写入使用。

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::FromRow;

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct DonationUrl {
    pub id: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct License {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct ModeratorMessage {
    pub message: String,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct GalleryItem {
    pub url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub ordering: Option<i64>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Project {
    #[serde(alias = "_id")]
    pub id: String,
    pub slug: String,
    pub title: Option<String>,
    pub description: Option<String>,
    #[sqlx(skip)]
    pub categories: Option<Vec<String>>,
    pub client_side: Option<String>,
    pub server_side: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    pub requested_status: Option<String>,
    #[sqlx(skip)]
    pub additional_categories: Option<Vec<String>>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    #[sqlx(skip)]
    pub donation_urls: Option<Vec<DonationUrl>>,
    pub project_type: Option<String>,
    pub downloads: Option<i64>,
    pub icon_url: Option<String>,
    /// Modrinth 中为 u32，数据库以 bigint 存储
    pub color: Option<i64>,
    pub thread_id: Option<String>,
    pub monetization_status: Option<String>,
    pub team: String,
    pub body_url: Option<String>,
    #[sqlx(skip)]
    pub moderator_message: Option<ModeratorMessage>,
    pub translated_description: Option<String>,
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub approved: Option<DateTime<Utc>>,
    pub queued: Option<DateTime<Utc>>,
    #[sqlx(try_from = "i64")]
    pub followers: u32,
    #[sqlx(skip)]
    pub license: Option<License>,
    #[sqlx(skip)]
    pub versions: Option<Vec<String>>,
    #[sqlx(skip)]
    pub game_versions: Option<Vec<String>>,
    #[sqlx(skip)]
    pub loaders: Option<Vec<String>>,
    #[sqlx(skip)]
    pub gallery: Option<Vec<GalleryItem>>,

    pub sync_at: DateTime<Utc>,
    pub checked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Dependencies {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Hashes {
    pub sha512: String,
    pub sha1: String,
}

/// `modrinth_files` 集合中的文件文档，MongoDB `_id` 为 `{sha512, sha1}`
#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct File {
    #[serde(alias = "_id")]
    #[sqlx(flatten)]
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<String>,
    pub version_id: String,
    pub project_id: String,
    pub file_cdn_cached: Option<bool>,
    pub found: Option<bool>,

    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct FileInfo {
    #[sqlx(flatten)]
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<String>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Version {
    #[serde(alias = "_id")]
    pub id: String,
    pub project_id: String,
    pub slug: Option<String>,
    pub name: Option<String>,
    pub version_number: Option<String>,
    pub changelog: Option<String>,
    #[sqlx(skip)]
    pub dependencies: Option<Vec<Dependencies>>,
    #[sqlx(skip)]
    pub game_versions: Option<Vec<String>>,
    pub version_type: Option<String>,
    #[sqlx(skip)]
    pub loaders: Option<Vec<String>>,
    pub featured: Option<bool>,
    pub status: Option<String>,
    pub requested_status: Option<String>,
    pub author_id: String,
    pub date_published: DateTime<Utc>,
    pub downloads: i64,
    pub changelog_url: Option<String>,
    #[sqlx(skip)]
    pub files: Vec<FileInfo>,
    pub found: Option<bool>,

    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Category {
    pub icon: String,
    pub name: String,
    pub project_type: Option<String>,
    pub header: String,

    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct Loader {
    pub icon: String,
    pub name: String,
    #[sqlx(skip)]
    pub supported_project_types: Vec<String>,
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone, FromRow)]
pub struct GameVersion {
    pub version: String,
    pub version_type: String,
    pub date: DateTime<Utc>,
    pub major: bool,

    pub sync_at: DateTime<Utc>,
}
