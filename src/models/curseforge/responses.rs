use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::curseforge::entities::{Author, FileIndex, Links, Logo, ScreenShot};

use crate::models::curseforge::entities::{Category, CategoryInfo, File, FileInfo, Mod};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FingerprintResult {
    #[serde(rename = "isCacheBuilt")]
    pub is_cache_built: bool,
    #[serde(rename = "exactMatches")]
    pub exact_matches: Vec<SingleFingerprintResponse>,
    #[serde(rename = "exactFingerprints")]
    pub exact_fingerprints: Vec<i64>,
    #[serde(rename = "installedFingerprints")]
    pub installed_fingerprints: Vec<i64>,
    #[serde(rename = "unmatchedFingerprints")]
    pub unmatched_fingerprints: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SingleFingerprintResponse {
    pub id: i32,
    pub file: FileInfo,
    #[serde(rename = "latestFiles")]
    pub latest_files: Vec<FileInfo>,

    pub sync_at: DateTime<Utc>,
}

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct _FileSortableGameVersions {
//     #[serde(rename = "gameVersionName")]
//     pub game_version_name: Option<String>,
//     #[serde(rename = "gameVersionPadded")]
//     pub game_version_padded: Option<String>,
//     #[serde(rename = "gameVersion")]
//     pub game_version: Option<String>,
//     #[serde(rename = "gameVersionReleaseDate")]
//     pub game_version_release_date: Option<DateTime<Utc>>,
//     #[serde(rename = "gameVersionTypeId")]
//     pub game_version_type_id: Option<i32>,
// }

// impl From<FileSortableGameVersions> for _FileSortableGameVersions{
//     fn from(file_sortable: FileSortableGameVersions) -> Self {
//         Self {
//             game_version_name: file_sortable.game_version_name,
//             game_version_padded: file_sortable.game_version_padded,
//             game_version: file_sortable.game_version,
//             game_version_release_date: file_sortable.game_version_release_date,
//             game_version_type_id: file_sortable.game_version_type_id,
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchResponse {
    // pub data: Vec<ModInfo>,
    pub data: Vec<ModInfo>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Pagination {
    pub index: i32,
    pub page_size: i32,
    pub result_count: i32,
    pub total_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModInfo {
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
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DownloadUrlResponse {
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModResponse {
    // pub data: ModReponseObject,
    pub data: Mod,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModsResponse {
    // pub data: Vec<ModReponseObject>,
    pub data: Vec<Mod>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModFilesResponse {
    // pub data: Vec<FileResponseObject>,
    pub data: Vec<File>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FileResponse {
    // pub data: FileResponseObject,
    pub data: File,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FilesResponse {
    // pub data: Vec<FileResponseObject>,
    pub data: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FingerprintResponse {
    pub data: FingerprintResult,
}

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct FileResponseObject {
//     pub id: i32,
//     #[serde(rename = "gameId")]
//     pub game_id: i32,
//     #[serde(rename = "modId")]
//     pub mod_id: i32,
//     #[serde(rename = "isAvailable")]
//     pub is_available: Option<bool>,
//     #[serde(rename = "displayName")]
//     pub display_name: Option<String>,
//     #[serde(rename = "fileName")]
//     pub file_name: Option<String>,
//     #[serde(rename = "releaseType")]
//     pub release_type: Option<i32>,
//     #[serde(rename = "fileStatus")]
//     pub file_status: Option<i32>,
//     pub hashes: Option<Vec<Hash>>,
//     #[serde(rename = "fileDate")]
//     pub file_date: Option<DateTime<Utc>>,
//     #[serde(rename = "fileLength")]
//     pub file_length: Option<i64>,
//     #[serde(rename = "downloadCount")]
//     pub download_count: Option<i64>,
//     #[serde(rename = "fileSizeOnDisk")]
//     pub file_size_on_disk: Option<i64>,
//     #[serde(rename = "downloadUrl")]
//     pub download_url: Option<String>,
//     #[serde(rename = "gameVersions")]
//     pub game_versions: Option<Vec<String>>,
//     #[serde(rename = "sortableGameVersions")]
//     pub sortable_game_versions: Option<Vec<FileSortableGameVersions>>,
//     pub dependencies: Option<Vec<FileDependencies>>,
//     #[serde(rename = "exposeAsAlternative")]
//     pub expose_as_alternative: Option<bool>,
//     #[serde(rename = "parentProjectFileId")]
//     pub parent_project_file_id: Option<i32>,
//     #[serde(rename = "alternateFileId")]
//     pub alternate_file_id: Option<i32>,
//     #[serde(rename = "isServerPack")]
//     pub is_server_pack: Option<bool>,
//     #[serde(rename = "serverPackFileId")]
//     pub server_pack_file_id: Option<i32>,
//     #[serde(rename = "isEarlyAccessContent")]
//     pub is_early_access_content: Option<bool>,
//     #[serde(rename = "earlyAccessEndDate")]
//     pub early_access_end_date: Option<DateTime<Utc>>,
//     #[serde(rename = "fileFingerprint")]
//     pub file_fingerprint: Option<i64>,
//     pub modules: Option<Vec<Module>>,

//     pub sync_at: DateTime<Utc>,
// }

// impl From<File> for FileResponseObject {
//     fn from(file: File) -> Self {
//         Self {
//             id: file.id,
//             game_id: file.game_id,
//             mod_id: file.mod_id,
//             is_available: file.is_available,
//             display_name: file.display_name,
//             file_name: file.file_name,
//             release_type: file.release_type,
//             file_status: file.file_status,
//             hashes: file.hashes,
//             file_date: file.file_date,
//             file_length: file.file_length,
//             download_count: file.download_count,
//             file_size_on_disk: file.file_size_on_disk,
//             download_url: file.download_url,
//             game_versions: file.game_versions,
//             sortable_game_versions: file.sortable_game_versions.map(|versions| versions.into_iter().map(|v| v.into()).collect()),
//             dependencies: file.dependencies,
//             expose_as_alternative: file.expose_as_alternative,
//             parent_project_file_id: file.parent_project_file_id,
//             alternate_file_id: file.alternate_file_id,
//             is_server_pack: file.is_server_pack,
//             server_pack_file_id: file.server_pack_file_id,
//             is_early_access_content: file.is_early_access_content,
//             early_access_end_date: file.early_access_end_date,
//             file_fingerprint: file.file_fingerprint,
//             modules: file.modules,

//             sync_at: file.sync_at,
//         }
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct _FileInfo {
//     pub id: i32,
//     #[serde(rename = "gameId")]
//     pub game_id: i32,
//     #[serde(rename = "modId")]
//     pub mod_id: i32,
//     #[serde(rename = "isAvailable")]
//     pub is_available: Option<bool>,
//     #[serde(rename = "displayName")]
//     pub display_name: Option<String>,
//     #[serde(rename = "fileName")]
//     pub file_name: Option<String>,
//     #[serde(rename = "releaseType")]
//     pub release_type: Option<i32>,
//     #[serde(rename = "fileStatus")]
//     pub file_status: Option<i32>,
//     pub hashes: Option<Vec<Hash>>,
//     #[serde(rename = "fileDate")]
//     pub file_date: Option<DateTime<Utc>>,
//     #[serde(rename = "fileLength")]
//     pub file_length: Option<i64>,
//     #[serde(rename = "downloadCount")]
//     pub download_count: Option<i64>,
//     #[serde(rename = "fileSizeOnDisk")]
//     pub file_size_on_disk: Option<i64>,
//     #[serde(rename = "downloadUrl")]
//     pub download_url: Option<String>,
//     #[serde(rename = "gameVersions")]
//     pub game_versions: Option<Vec<String>>,
//     #[serde(rename = "sortableGameVersions")]
//     pub sortable_game_versions: Option<Vec<FileSortableGameVersions>>,
//     pub dependencies: Option<Vec<FileDependencies>>,
//     #[serde(rename = "exposeAsAlternative")]
//     pub expose_as_alternative: Option<bool>,
//     #[serde(rename = "parentProjectFileId")]
//     pub parent_project_file_id: Option<i32>,
//     #[serde(rename = "alternateFileId")]
//     pub alternate_file_id: Option<i32>,
//     #[serde(rename = "isServerPack")]
//     pub is_server_pack: Option<bool>,
//     #[serde(rename = "serverPackFileId")]
//     pub server_pack_file_id: Option<i32>,
//     #[serde(rename = "isEarlyAccessContent")]
//     pub is_early_access_content: Option<bool>,
//     #[serde(rename = "earlyAccessEndDate")]
//     pub early_access_end_date: Option<DateTime<Utc>>,
//     #[serde(rename = "fileFingerprint")]
//     pub file_fingerprint: Option<i64>,
//     pub modules: Option<Vec<Module>>,
// }

// impl From<FileInfo> for _FileInfo {
//     fn from(file: FileInfo) -> Self {
//         Self {
//             id: file.id,
//             game_id: file.game_id,
//             mod_id: file.mod_id,
//             is_available: file.is_available,
//             display_name: file.display_name,
//             file_name: file.file_name,
//             release_type: file.release_type,
//             file_status: file.file_status,
//             hashes: file.hashes,
//             file_date: file.file_date,
//             file_length: file.file_length,
//             download_count: file.download_count,
//             file_size_on_disk: file.file_size_on_disk,
//             download_url: file.download_url,
//             game_versions: file.game_versions,
//             sortable_game_versions: file.sortable_game_versions.map(|versions| versions.into_iter().map(|v| v.into()).collect()),
//             dependencies: file.dependencies,
//             expose_as_alternative: file.expose_as_alternative,
//             parent_project_file_id: file.parent_project_file_id,
//             alternate_file_id: file.alternate_file_id,
//             is_server_pack: file.is_server_pack,
//             server_pack_file_id: file.server_pack_file_id,
//             is_early_access_content: file.is_early_access_content,
//             early_access_end_date: file.early_access_end_date,
//             file_fingerprint: file.file_fingerprint,
//             modules: file.modules,
//         }
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct ModReponseObject {
//     pub id: i32,
//     #[serde(rename = "gameId")]
//     pub game_id: Option<i32>,
//     pub name: Option<String>,
//     pub slug: String,
//     pub links: Option<Links>,
//     pub summary: Option<String>,
//     pub status: Option<i32>,
//     #[serde(rename = "downloadCount")]
//     pub download_count: Option<i64>,
//     #[serde(rename = "isFeatured")]
//     pub is_featured: Option<bool>,
//     #[serde(rename = "primaryCategoryId")]
//     pub primary_category_id: Option<i32>,
//     pub categories: Option<Vec<CategoryInfo>>,
//     #[serde(rename = "classId")]
//     pub class_id: Option<i32>,
//     pub authors: Option<Vec<Author>>,
//     pub logo: Option<Logo>,
//     pub screenshots: Option<Vec<ScreenShot>>,
//     #[serde(rename = "mainFileId")]
//     pub main_file_id: Option<i32>,
//     #[serde(rename = "latestFiles")]
//     pub latest_files: Option<Vec<_FileInfo>>,
//     #[serde(rename = "latestFilesIndexes")]
//     pub latest_files_indexes: Option<Vec<FileIndex>>,
//     #[serde(rename = "dateCreated")]
//     pub date_created: Option<DateTime<Utc>>,
//     #[serde(rename = "dateModified")]
//     pub date_modified: Option<DateTime<Utc>>,
//     #[serde(rename = "dateReleased")]
//     pub date_released: Option<DateTime<Utc>>,
//     #[serde(rename = "allowModDistribution")]
//     pub allow_mod_distribution: Option<bool>,
//     #[serde(rename = "gamePopularityRank")]
//     pub game_popularity_rank: Option<i32>,
//     #[serde(rename = "isAvailable")]
//     pub is_available: Option<bool>,
//     #[serde(rename = "thumbsUpCount")]
//     pub thumbs_up_count: Option<i32>,
//     pub rating: Option<i32>,

//     pub sync_at: DateTime<Utc>,
// }

// impl From<Mod> for ModReponseObject {
//     fn from(mod_info: Mod) -> Self {
//         Self {
//             id: mod_info.id,
//             game_id: mod_info.game_id,
//             name: mod_info.name,
//             slug: mod_info.slug,
//             links: mod_info.links,
//             summary: mod_info.summary,
//             status: mod_info.status,
//             download_count: mod_info.download_count,
//             is_featured: mod_info.is_featured,
//             primary_category_id: mod_info.primary_category_id,
//             categories: mod_info.categories.map(|cats| cats.into_iter().map(|cat| cat.into()).collect()),
//             class_id: mod_info.class_id,
//             authors: mod_info.authors,
//             logo: mod_info.logo,
//             screenshots: mod_info.screenshots,
//             main_file_id: mod_info.main_file_id,
//             latest_files: mod_info.latest_files.map(|files| files.into_iter().map(|file| file.into()).collect()),
//             latest_files_indexes: mod_info.latest_files_indexes,
//             date_created: mod_info.date_created,
//             date_modified: mod_info.date_modified,
//             date_released: mod_info.date_released,
//             allow_mod_distribution: mod_info.allow_mod_distribution,
//             game_popularity_rank: mod_info.game_popularity_rank,
//             is_available: mod_info.is_available,
//             thumbs_up_count: mod_info.thumbs_up_count,
//             rating: mod_info.rating,

//             sync_at: mod_info.sync_at,
//         }
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct FingerprintResponseObject {
//     pub id: i32,
//     pub file: FileInfo,
//     #[serde(rename = "latestFiles")]
//     pub latest_files: Vec<FileInfo>,

//     pub sync_at: DateTime<Utc>,
// }

// impl From<Fingerprint> for FingerprintResponseObject {
//     fn from(fingerprint: Fingerprint) -> Self {
//         Self {
//             id: fingerprint.id,
//             file: fingerprint.file.into(),
//             latest_files: fingerprint.latest_files.into_iter().map(|file| file.into()).collect(),

//             sync_at: fingerprint.sync_at,
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CategoriesResponse {
    // pub data: Vec<CategoryResponseObject>,
    pub data: Vec<Category>,
}

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// #[serde(rename_all = "camelCase")]
// pub struct _CategoryInfo {
//     pub id: Option<i32>,
//     #[serde(rename = "gameId")]
//     pub game_id: Option<i32>,
//     pub name: Option<String>,
//     pub slug: Option<String>,
//     pub url: Option<String>,
//     #[serde(rename = "iconUrl")]
//     pub icon_url: Option<String>,
//     #[serde(rename = "dateModified")]
//     pub date_modified: Option<DateTime<Utc>>,
//     #[serde(rename = "isClass")]
//     pub is_class: Option<bool>,
//     #[serde(rename = "classId")]
//     pub class_id: Option<i32>,
//     #[serde(rename = "parentCategoryId")]
//     pub parent_category_id: Option<i32>,
//     #[serde(rename = "displayIndex")]
//     pub display_index: Option<i32>,
// }

// #[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
// pub struct CategoryResponseObject {
//     pub id: i32,
//     #[serde(rename = "gameId")]
//     pub game_id: i32,
//     pub name: String,
//     pub slug: Option<String>,
//     pub url: String,
//     #[serde(rename = "iconUrl")]
//     pub icon_url: String,
//     #[serde(rename = "dateModified")]
//     pub date_modified: String,
//     #[serde(rename = "isClass")]
//     pub is_class: Option<bool>,
//     #[serde(rename = "classId")]
//     pub class_id: Option<i32>,
//     #[serde(rename = "parentCategoryId")]
//     pub parent_category_id: Option<i32>,
//     #[serde(rename = "displayIndex")]
//     pub display_index: i32,

//     pub sync_at: DateTime<Utc>,
// }

// impl From<Category> for CategoryResponseObject {
//     fn from(category: Category) -> Self {
//         Self {
//             id: category.id,
//             game_id: category.game_id,
//             name: category.name,
//             slug: category.slug,
//             url: category.url,
//             icon_url: category.icon_url,
//             date_modified: category.date_modified,
//             is_class: category.is_class,
//             class_id: category.class_id,
//             parent_category_id: category.parent_category_id,
//             display_index: category.display_index,

//             sync_at: category.sync_at,
//         }
//     }
// }

// impl From<CategoryInfo> for _CategoryInfo {
//     fn from(category: CategoryInfo) -> Self {
//         Self {
//             id: category.id,
//             game_id: category.game_id,
//             name: category.name,
//             slug: category.slug,
//             url: category.url,
//             icon_url: category.icon_url,
//             date_modified: category.date_modified,
//             is_class: category.is_class,
//             class_id: category.class_id,
//             parent_category_id: category.parent_category_id,
//             display_index: category.display_index,
//         }
//     }
// }
