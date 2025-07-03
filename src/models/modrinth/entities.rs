use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema};


use bson::serde_helpers::{chrono_datetime_as_bson_datetime, chrono_datetime_as_bson_datetime_optional};


#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DonationUrl {
    pub id: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct License {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GalleryItem {
    pub url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub created: DateTime<Utc>,
    pub ordering: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Project {
    #[serde(alias = "_id")]
    pub id: String,
    pub slug: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub client_side: Option<String>,
    pub server_side: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    pub requested_status: Option<String>,
    pub additional_categories: Option<Vec<String>>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Option<Vec<DonationUrl>>,
    pub project_type: Option<String>,
    pub downloads: Option<i64>,
    pub icon_url: Option<String>,
    pub color: Option<i32>,
    pub thread_id: Option<String>,
    pub monetization_status: Option<String>,
    pub team: String,
    pub body_url: Option<String>,
    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub published: DateTime<Utc>,
    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub updated: DateTime<Utc>,
    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime_optional::deserialize")]
    pub approved: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime_optional::deserialize")]
    pub queued: Option<DateTime<Utc>>,
    pub followers: i32,
    pub license: Option<License>,
    pub versions: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub loaders: Option<Vec<String>>,
    pub gallery: Option<Vec<GalleryItem>>,

    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Dependencies {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Hashes {
    pub sha512: String,
    pub sha1: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct File {
    #[serde(alias = "_id")]
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<String>,
    pub version_id: String,
    pub project_id: String,
    pub file_cdn_cached: Option<bool>,

    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FileInfo {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Version {
    #[serde(alias = "_id")]
    pub id: String,
    pub project_id: String,
    pub name: Option<String>,
    pub version_number: Option<String>,
    pub changelog: Option<String>,
    pub dependencies: Option<Vec<Dependencies>>,
    pub game_versions: Option<Vec<String>>,
    pub version_type: Option<String>,
    pub loaders: Option<Vec<String>>,
    pub featured: Option<bool>,
    pub status: Option<String>,
    pub requested_status: Option<String>,
    pub author_id: String,
    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub date_published: DateTime<Utc>,
    pub downloads: i64,
    pub changelog_url: Option<String>,
    pub files: Vec<FileInfo>,

    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Category {
    pub icon: String,
    pub name: String,
    pub project_type: Option<String>,
    pub header: String,

    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Loader {
    pub icon: String,
    pub name: String,
    pub supported_project_types: Vec<String>,
    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GameVersion {
    pub version: String,
    pub version_type: String,
    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub date: DateTime<Utc>,
    pub major: bool,

    #[serde(deserialize_with = "chrono_datetime_as_bson_datetime::deserialize")]
    pub sync_at: DateTime<Utc>,
}