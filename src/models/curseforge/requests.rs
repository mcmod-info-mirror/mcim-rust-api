use utoipa::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct SearchQuery {
    #[serde(rename = "gameId")]
    pub game_id: Option<i32>,
    #[serde(rename = "classId")]
    pub class_id: Option<i32>,
    #[serde(rename = "categoryId")]
    pub category_id: Option<i32>,
    #[serde(rename = "categoryIds")]
    pub category_ids: Option<String>,
    #[serde(rename = "gameVersion")]
    pub game_version: Option<String>,
    #[serde(rename = "gameVersions")]
    pub game_versions: Option<String>,
    #[serde(rename = "searchFilter")]
    pub search_filter: Option<String>,
    #[serde(rename = "sortField")]
    pub sort_field: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<String>,
    #[serde(rename = "modLoaderType")]
    pub mod_loader_type: Option<String>,
    #[serde(rename = "modLoaderTypes")]
    pub mod_loader_types: Option<String>,
    #[serde(rename = "gameVersionTypeId")]
    pub game_version_type_id: Option<i32>,
    #[serde(rename = "authorId")]
    pub author_id: Option<i32>,
    #[serde(rename = "primaryAuthorId")]
    pub primary_author_id: Option<i32>,
    pub slug: Option<String>,
    pub index: Option<i32>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ModsBody {
    #[serde(rename = "modIds")]
    pub mod_ids: Vec<i32>,
    #[serde(rename = "filterPcOnly")]
    pub filter_pc_only: Option<bool>,
}


#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ModFilesQuery {
    #[serde(rename = "gameVersion")]
    pub game_version: Option<String>,
    #[serde(rename = "modLoaderType")]
    pub mod_loader_type: Option<i32>,
    pub index: Option<i32>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<i32>,
}


#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct FileIdsBody {
    #[serde(rename = "fileIds")]
    pub file_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct FingerprintsBody {
    pub fingerprints: Vec<i32>
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CategoriesQuery {
    #[serde(rename = "gameId")]
    pub game_id: i32,
    #[serde(rename = "classId")]
    pub class_id: Option<i32>,
    #[serde(rename = "classOnly")]
    pub class_only: Option<bool>,
}