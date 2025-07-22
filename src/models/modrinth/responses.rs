use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use crate::models::modrinth::entities as db;

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
    pub created: DateTime<Utc>,
    pub ordering: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Project {
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
    pub published: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub approved: Option<DateTime<Utc>>,
    pub queued: Option<DateTime<Utc>>,
    pub followers: u32,
    pub license: Option<License>,
    pub versions: Option<Vec<String>>,
    pub game_versions: Option<Vec<String>>,
    pub loaders: Option<Vec<String>>,
    pub gallery: Option<Vec<GalleryItem>>,

    #[serde(default = "Utc::now")]
    pub sync_at: DateTime<Utc>,
}

impl From<db::Project> for Project {
    fn from(project: db::Project) -> Self {
        Project {
            id: project.id,
            slug: project.slug,
            title: project.title,
            description: project.description,
            categories: project.categories,
            client_side: project.client_side,
            server_side: project.server_side,
            body: project.body,
            status: project.status,
            requested_status: project.requested_status,
            additional_categories: project.additional_categories,
            issues_url: project.issues_url,
            source_url: project.source_url,
            wiki_url: project.wiki_url,
            discord_url: project.discord_url,
            donation_urls: project
                .donation_urls
                .map(|urls| urls.into_iter().map(Into::into).collect()),
            project_type: project.project_type,
            downloads: project.downloads,
            icon_url: project.icon_url,
            color: project.color,
            thread_id: project.thread_id,
            monetization_status: project.monetization_status,
            team: project.team,
            body_url: project.body_url,
            published: project.published,
            updated: project.updated,
            approved: project.approved,
            queued: project.queued,
            followers: project.followers,
            license: project.license.map(Into::into),
            versions: project
                .versions
                .map(|v| v.into_iter().map(String::from).collect()),
            game_versions: project
                .game_versions
                .map(|v| v.into_iter().map(String::from).collect()),
            loaders: project
                .loaders
                .map(|l| l.into_iter().map(String::from).collect()),
            gallery: project
                .gallery
                .map(|gallery_vec| gallery_vec.into_iter().map(Into::into).collect()),

            sync_at: project.sync_at,
        }
    }
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
pub struct FileInfo {
    pub hashes: Hashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub file_type: Option<String>,
}

impl From<db::FileInfo> for FileInfo {
    fn from(file_info: db::FileInfo) -> Self {
        FileInfo {
            hashes: file_info.hashes.into(),
            url: file_info.url,
            filename: file_info.filename,
            primary: file_info.primary,
            size: file_info.size,
            file_type: file_info.file_type,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Version {
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
    pub date_published: DateTime<Utc>,
    pub downloads: i64,
    pub changelog_url: Option<String>,
    pub files: Vec<FileInfo>,

    #[serde(default = "Utc::now")]
    pub sync_at: DateTime<Utc>,
}

impl From<db::Version> for Version {
    fn from(version: db::Version) -> Self {
        Version {
            id: version.id,
            project_id: version.project_id,
            name: version.name,
            version_number: version.version_number,
            changelog: version.changelog,
            dependencies: version
                .dependencies
                .map(|deps| deps.into_iter().map(Into::into).collect()),
            game_versions: version
                .game_versions
                .map(|v| v.into_iter().map(String::from).collect()),
            version_type: version.version_type,
            loaders: version
                .loaders
                .map(|l| l.into_iter().map(String::from).collect()),
            featured: version.featured,
            status: version.status,
            requested_status: version.requested_status,
            author_id: version.author_id,
            date_published: version.date_published,
            downloads: version.downloads,
            changelog_url: version.changelog_url,
            files: version.files.into_iter().map(Into::into).collect(),

            sync_at: version.sync_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Category {
    pub icon: String,
    pub name: String,
    pub project_type: Option<String>,
    pub header: String,

    #[serde(default = "Utc::now")]
    pub sync_at: DateTime<Utc>,
}

impl From<db::Category> for Category {
    fn from(category: db::Category) -> Self {
        Category {
            icon: category.icon,
            name: category.name,
            project_type: category.project_type,
            header: category.header,

            sync_at: category.sync_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Loader {
    pub icon: String,
    pub name: String,
    pub supported_project_types: Vec<String>,

    #[serde(default = "Utc::now")]
    pub sync_at: DateTime<Utc>,
}

impl From<db::Loader> for Loader {
    fn from(loader: db::Loader) -> Self {
        Loader {
            icon: loader.icon,
            name: loader.name,
            supported_project_types: loader.supported_project_types,

            sync_at: loader.sync_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GameVersion {
    pub version: String,
    pub version_type: String,
    pub date: DateTime<Utc>,
    pub major: bool,

    #[serde(default = "Utc::now")]
    pub sync_at: DateTime<Utc>,
}

impl From<db::GameVersion> for GameVersion {
    fn from(game_version: db::GameVersion) -> Self {
        GameVersion {
            version: game_version.version,
            version_type: game_version.version_type,
            date: game_version.date,
            major: game_version.major,

            sync_at: game_version.sync_at,
        }
    }
}

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

impl_from_db!(db::DonationUrl, DonationUrl);
impl_from_db!(db::License, License);
impl_from_db!(db::GalleryItem, GalleryItem);
impl_from_db!(db::Dependencies, Dependencies);
impl_from_db!(db::Hashes, Hashes);

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct MutilFilesResponse {
    #[serde(flatten)]
    pub entries: Option<HashMap<String, Version>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CategoriesResponse {
    #[serde(flatten)]
    pub categories: Vec<Category>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LoadersResponse {
    #[serde(flatten)]
    pub loaders: Vec<Loader>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GameVersionsResponse {
    #[serde(flatten)]
    pub game_versions: Vec<GameVersion>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchHit {
    pub project_id: String,
    pub project_type: String,
    pub slug: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub display_categories: Option<Vec<String>>,
    pub versions: Vec<String>,
    pub downloads: i64,
    pub follows: u32,
    pub icon_url: String,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub latest_version: Option<String>,
    pub license: String,
    pub client_side: String,
    pub server_side: String,
    pub gallery: Option<Vec<String>>,
    pub featured_gallery: Option<String>,
    pub color: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub offset: i32,
    pub limit: i32,
    pub total_hits: i32,
}

// ...existing code...

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use std::env;

    // ...existing helper functions...

    fn get_base_url() -> String {
        dotenv().ok();
        env::var("MODRINTH_API_ENDPOINT").unwrap_or_else(|_| "https://api.modrinth.com".to_string())
    }

    // ...existing unit tests...

    #[tokio::test]
    async fn test_search_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();

        let response = client
            .get(&format!("{}/v2/search?query=sodium", base_url))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let search_response: SearchResponse = response
            .json()
            .await
            .expect("Failed to deserialize SearchResponse");

        assert!(!search_response.hits.is_empty());
    }

    #[tokio::test]
    async fn test_project_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();

        let response = client
            .get(&format!("{}/v2/project/Wnxd13zP", base_url))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let project: Project = response
            .json()
            .await
            .expect("Failed to deserialize Project");

        assert_eq!(project.id, "Wnxd13zP");
    }

    #[tokio::test]
    async fn test_projects_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let project_ids = vec!["Wnxd13zP", "Ua7DFN59"];

        let response = client
            .get(&format!("{}/v2/projects?ids={:?}", base_url, project_ids))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let projects: Vec<Project> = response
            .json()
            .await
            .expect("Failed to deserialize Vec<Project>");

        assert_eq!(projects.len(), 2);
    }

    #[tokio::test]
    async fn test_version_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();

        let response = client
            .get(&format!("{}/v2/version/dpSzBMP6", base_url))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let version: Version = response
            .json()
            .await
            .expect("Failed to deserialize Version");

        assert_eq!(version.id, "dpSzBMP6");
    }

    #[tokio::test]
    async fn test_versions_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let version_ids = vec!["dpSzBMP6", "IOIGqCVr"];

        let response = client
            .get(&format!("{}/v2/versions?ids={:?}", base_url, version_ids))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let versions: Vec<Version> = response
            .json()
            .await
            .expect("Failed to deserialize Vec<Version>");

        assert_eq!(versions.len(), 2);
    }

    #[tokio::test]
    async fn test_project_versions_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();

        let response = client
            .get(&format!("{}/v2/project/Wnxd13zP/version", base_url))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let versions: Vec<Version> = response
            .json()
            .await
            .expect("Failed to deserialize Vec<Version>");

        assert!(!versions.is_empty());
    }

    #[tokio::test]
    async fn test_version_file_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();

        let response = client
            .get(&format!(
                "{}/v2/version_file/f0cea90219f681c3183e0d37d021cb8902d2d085?algorithm=sha1",
                base_url
            ))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let version: Version = response
            .json()
            .await
            .expect("Failed to deserialize Version");

        assert!(!version.files.is_empty());
        assert_eq!(version.id, "3AH29I4c");
    }

    #[tokio::test]
    async fn test_version_files_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "algorithm": "sha1",
            "hashes": ["f0cea90219f681c3183e0d37d021cb8902d2d085", "627c93adb68e04ffb390ad0e5dbf62d342f27a28"]
        });

        let response = client
            .post(&format!("{}/v2/version_files", base_url))
            .json(&payload)
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let multi_files: MutilFilesResponse = response
            .json()
            .await
            .expect("Failed to deserialize MutilFilesResponse");

        if let Some(entries) = multi_files.entries {
            assert!(!entries.is_empty());
            assert!(entries.contains_key("f0cea90219f681c3183e0d37d021cb8902d2d085"));
            assert!(entries.contains_key("627c93adb68e04ffb390ad0e5dbf62d342f27a28"));
        }
    }

    #[tokio::test]
    async fn test_categories_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();

        let response = client
            .get(&format!("{}/v2/tag/category", base_url))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let categories: Vec<Category> = response
            .json()
            .await
            .expect("Failed to deserialize Vec<Category>");

        assert!(!categories.is_empty());
    }

    #[tokio::test]
    async fn test_loaders_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();

        let response = client
            .get(&format!("{}/v2/tag/loader", base_url))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let loaders: Vec<Loader> = response
            .json()
            .await
            .expect("Failed to deserialize Vec<Loader>");

        assert!(!loaders.is_empty());
    }

    #[tokio::test]
    async fn test_game_versions_response() {
        let base_url = get_base_url();
        let client = reqwest::Client::new();

        let response = client
            .get(&format!("{}/v2/tag/game_version", base_url))
            .send()
            .await
            .expect("Failed to send request");

        assert!(response.status().is_success());

        let game_versions: Vec<GameVersion> = response
            .json()
            .await
            .expect("Failed to deserialize Vec<GameVersion>");

        assert!(!game_versions.is_empty());
    }
}
