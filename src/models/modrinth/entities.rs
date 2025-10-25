use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_with::serde_as;

use bson::serde_helpers::datetime::FromChrono04DateTime;

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct DonationUrl {
    pub id: Option<String>,
    pub platform: Option<String>,
    pub url: Option<String>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct License {
    pub id: Option<String>,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct GalleryItem {
    pub url: String,
    pub featured: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde_as(as = "FromChrono04DateTime")]
    pub created: DateTime<Utc>,
    pub ordering: Option<i64>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
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
    pub color: Option<u32>,
    pub thread_id: Option<String>,
    pub monetization_status: Option<String>,
    pub team: String,
    pub body_url: Option<String>,
    #[serde_as(as = "FromChrono04DateTime")]
    pub published: DateTime<Utc>,
    #[serde_as(as = "FromChrono04DateTime")]
    pub updated: DateTime<Utc>,
    #[serde_as(as = "Option<FromChrono04DateTime>")]
    pub approved: Option<DateTime<Utc>>,
    #[serde_as(as = "Option<FromChrono04DateTime>")]
    pub queued: Option<DateTime<Utc>>,
    pub followers: u32,
    pub license: Option<License>,
    pub versions: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub loaders: Option<Vec<String>>,
    pub gallery: Option<Vec<GalleryItem>>,

    #[serde_as(as = "FromChrono04DateTime")]
    pub sync_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct Dependencies {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub file_name: Option<String>,
    pub dependency_type: String,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct Hashes {
    pub sha512: String,
    pub sha1: String,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
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

    #[serde_as(as = "FromChrono04DateTime")]
    pub sync_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct FileInfo {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<String>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
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
    #[serde_as(as = "FromChrono04DateTime")]
    pub date_published: DateTime<Utc>,
    pub downloads: i64,
    pub changelog_url: Option<String>,
    pub files: Vec<FileInfo>,

    #[serde_as(as = "FromChrono04DateTime")]
    pub sync_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct Category {
    pub icon: String,
    pub name: String,
    pub project_type: Option<String>,
    pub header: String,

    #[serde_as(as = "FromChrono04DateTime")]
    pub sync_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct Loader {
    pub icon: String,
    pub name: String,
    pub supported_project_types: Vec<String>,
    #[serde_as(as = "FromChrono04DateTime")]
    pub sync_at: DateTime<Utc>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct GameVersion {
    pub version: String,
    pub version_type: String,
    #[serde_as(as = "FromChrono04DateTime")]
    pub date: DateTime<Utc>,
    pub major: bool,

    #[serde_as(as = "FromChrono04DateTime")]
    pub sync_at: DateTime<Utc>,
}
