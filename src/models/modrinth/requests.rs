use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthStatistics {
    pub projects: i32,
    pub versions: i32,
    pub files: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SearchIndex {
    #[serde(rename = "relevance")]
    Relevance,
    #[serde(rename = "downloads")]
    Downloads,
    #[serde(rename = "follows")]
    Follows,
    #[serde(rename = "newest")]
    Newest,
    #[serde(rename = "updated")]
    Updated,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub facets: Option<String>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
    pub index: Option<SearchIndex>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Algorithm {
    #[serde(rename = "sha1")]
    Sha1,
    #[serde(rename = "sha512")]
    Sha512,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HashesQuery {
    pub hashes: Vec<String>,
    pub algorithm: Algorithm,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateItems {
    pub loaders: Vec<String>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiUpdateItems {
    pub hashes: Vec<String>,
    pub algorithm: Algorithm,
    pub loaders: Option<Vec<String>>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectQuery {
    pub idslug: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectsQuery {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionQuery {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionsQuery {
    pub ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileQuery {
    pub hash: String,
    pub algorithm: Option<Algorithm>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileUpdateQuery {
    pub hash: String,
    pub algorithm: Option<Algorithm>,
    pub items: UpdateItems,
}

// Response wrappers
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModrinthResponse<T> {
    pub content: T,
    pub trustable: bool,
}

// Search result types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchHit {
    #[serde(rename = "project_id")]
    pub project_id: String,
    #[serde(rename = "project_type")]
    pub project_type: Option<String>,
    pub slug: Option<String>,
    pub author: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub categories: Option<Vec<String>>,
    pub versions: Option<Vec<String>>,
    pub downloads: Option<i64>,
    pub follows: Option<i32>,
    #[serde(rename = "icon_url")]
    pub icon_url: Option<String>,
    #[serde(rename = "date_created")]
    pub date_created: Option<String>,
    #[serde(rename = "date_modified")]
    pub date_modified: Option<String>,
    #[serde(rename = "latest_version")]
    pub latest_version: Option<String>,
    pub license: Option<String>,
    #[serde(rename = "client_side")]
    pub client_side: Option<String>,
    #[serde(rename = "server_side")]
    pub server_side: Option<String>,
    pub gallery: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub offset: i32,
    pub limit: i32,
    #[serde(rename = "total_hits")]
    pub total_hits: i32,
}

// File response types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileVersionResponse(pub HashMap<String, super::entities::Version>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MultiFileUpdateResponse(pub HashMap<String, super::entities::Version>);

// API endpoint parameter types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectEndpointParams {
    pub idslug: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectsEndpointParams {
    pub ids: String, // JSON encoded string array
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionEndpointParams {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionsEndpointParams {
    pub ids: String, // JSON encoded string array
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEndpointParams {
    pub hash: String,
    pub algorithm: Option<Algorithm>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchEndpointParams {
    pub query: Option<String>,
    pub facets: Option<String>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
    pub index: Option<SearchIndex>,
}