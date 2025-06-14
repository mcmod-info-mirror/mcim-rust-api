use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::deserialize_bson_datetime_flexible;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DonationUrl {
    pub id: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct License {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GalleryItem {
    pub url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub created: DateTime<Utc>,
    pub ordering: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: String,
    pub slug: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    #[serde(rename = "clientSide")]
    pub client_side: Option<String>,
    #[serde(rename = "serverSide")]
    pub server_side: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "requestedStatus")]
    pub requested_status: Option<String>,
    #[serde(rename = "additionalCategories")]
    pub additional_categories: Option<Vec<String>>,
    #[serde(rename = "issuesUrl")]
    pub issues_url: Option<String>,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
    #[serde(rename = "wikiUrl")]
    pub wiki_url: Option<String>,
    #[serde(rename = "discordUrl")]
    pub discord_url: Option<String>,
    #[serde(rename = "donationUrls")]
    pub donation_urls: Option<Vec<DonationUrl>>,
    #[serde(rename = "projectType")]
    pub project_type: Option<String>,
    pub downloads: Option<i64>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    pub color: Option<i32>,
    #[serde(rename = "threadId")]
    pub thread_id: Option<String>,
    #[serde(rename = "monetizationStatus")]
    pub monetization_status: Option<String>,
    pub team: String,
    #[serde(rename = "bodyUrl")]
    pub body_url: Option<String>,
    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub published: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub updated: DateTime<Utc>,
    #[serde(rename = "approved", deserialize_with = "deserialize_bson_datetime_flexible")]
    pub approved: Option<DateTime<Utc>>,
    #[serde(rename = "queued", deserialize_with = "deserialize_bson_datetime_flexible")]
    pub queued: Option<DateTime<Utc>>,
    pub followers: i32,
    pub license: Option<License>,
    pub versions: Option<Vec<String>>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Option<Vec<String>>,
    pub loaders: Option<Vec<String>>,
    pub gallery: Option<Vec<GalleryItem>>,

    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dependencies {
    #[serde(rename = "versionId")]
    pub version_id: Option<String>,
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(rename = "dependencyType")]
    pub dependency_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hashes {
    pub sha512: String,
    pub sha1: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct File {
    #[serde(rename = "_id")]
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    #[serde(rename = "fileType")]
    pub file_type: Option<String>,
    #[serde(rename = "versionId")]
    pub version_id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "fileCdnCached")]
    pub file_cdn_cached: Option<bool>,

    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    #[serde(rename = "fileType")]
    pub file_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Version {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub name: Option<String>,
    #[serde(rename = "versionNumber")]
    pub version_number: Option<String>,
    pub changelog: Option<String>,
    pub dependencies: Option<Vec<Dependencies>>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Option<Vec<String>>,
    #[serde(rename = "versionType")]
    pub version_type: Option<String>,
    pub loaders: Option<Vec<String>>,
    pub featured: Option<bool>,
    pub status: Option<String>,
    #[serde(rename = "requestedStatus")]
    pub requested_status: Option<String>,
    #[serde(rename = "authorId")]
    pub author_id: String,
    #[serde(rename = "datePublished", deserialize_with = "deserialize_bson_datetime_flexible")]
    pub date_published: DateTime<Utc>,
    pub downloads: i64,
    #[serde(rename = "changelogUrl")]
    pub changelog_url: Option<String>,
    pub files: Vec<FileInfo>,

    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    #[serde(rename = "_id")]
    pub icon: String,
    pub name: String,
    #[serde(rename = "projectType")]
    pub project_type: String,
    pub header: String,

    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Loader {
    #[serde(rename = "_id")]
    pub icon: String,
    pub name: String,
    #[serde(rename = "supportedProjectTypes")]
    pub supported_project_types: Vec<String>,

    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub sync_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameVersion {
    #[serde(rename = "_id")]
    pub version: String,
    #[serde(rename = "versionType")]
    pub version_type: String,
    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub date: DateTime<Utc>,
    pub major: bool,

    #[serde(deserialize_with = "deserialize_bson_datetime_flexible")]
    pub sync_at: DateTime<Utc>,
}