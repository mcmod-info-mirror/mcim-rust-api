use serde::{Deserialize, Serialize};
use super::entities::{Mod, File, Fingerprint, Category};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pagination {
    pub index: i32,
    pub page_size: i32,
    pub result_count: i32,
    pub total_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl Default for FingerprintResult {
    fn default() -> Self {
        Self {
            is_cache_built: true,
            exact_matches: Vec::new(),
            exact_fingerprints: Vec::new(),
            installed_fingerprints: Vec::new(),
            unmatched_fingerprints: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResponse {
    pub data: Vec<Mod>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadUrlResponse {
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModResponse {
    pub data: Mod,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModsResponse {
    pub data: Vec<Mod>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFilesResponse {
    pub data: Vec<File>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileResponse {
    pub data: File,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilesResponse {
    pub data: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FingerprintResponse {
    pub data: FingerprintResult,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CategoriesResponse {
    pub data: Vec<Category>,
}