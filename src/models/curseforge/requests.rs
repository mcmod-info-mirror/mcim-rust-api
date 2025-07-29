use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

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

impl Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SearchQuery {{ game_id: {:?}, class_id: {:?}, category_id: {:?}, category_ids: {:?}, game_version: {:?}, game_versions: {:?}, search_filter: {:?}, sort_field: {:?}, sort_order: {:?}, mod_loader_type: {:?}, mod_loader_types: {:?}, game_version_type_id: {:?}, author_id: {:?}, primary_author_id: {:?}, slug: {:?}, index: {:?}, page_size: {:?} }}",
            self.game_id, self.class_id, self.category_id, self.category_ids, self.game_version, self.game_versions, self.search_filter, self.sort_field, self.sort_order, self.mod_loader_type, self.mod_loader_types, self.game_version_type_id, self.author_id, self.primary_author_id, self.slug, self.index, self.page_size)
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ModsBody {
    #[serde(rename = "modIds")]
    #[schema(default = "[238222]")]
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
    #[schema(default = "[6614392]")]
    pub file_ids: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct FingerprintsBody {
    #[schema(default = "[510490952]")]
    pub fingerprints: Vec<i64>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CategoriesQuery {
    #[serde(rename = "gameId")]
    #[schema(default = 432)]
    pub game_id: i32,
    #[serde(rename = "classId")]
    #[schema(default = 6)]
    pub class_id: Option<i32>,
    #[serde(rename = "classOnly")]
    pub class_only: Option<bool>,
}
