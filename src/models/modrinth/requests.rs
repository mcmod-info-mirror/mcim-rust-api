use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub facets: Option<String>,
    pub offset: Option<i32>,
    pub limit: Option<i32>,
    pub index: Option<String>,
}

impl Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SearchQuery {{ query: {:?}, facets: {:?}, offset: {:?}, limit: {:?}, index: {:?} }}",
            self.query, self.facets, self.offset, self.limit, self.index
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct HashesQuery {
    #[schema(default = "[\"d67e66ea4bb2409997b636dae4203d33764cdcc8\"]")]
    pub hashes: Vec<String>,
    #[schema(default = "sha1")]
    pub algorithm: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateItems {
    #[schema(default = "[\"fabric\"]")]
    pub loaders: Vec<String>,
    #[schema(default = "[\"1.16.5\"]")]
    pub game_versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct MultiUpdateItems {
    #[schema(default = "[\"d67e66ea4bb2409997b636dae4203d33764cdcc8\"]")]
    pub hashes: Vec<String>,
    #[schema(default = "sha1")]
    pub algorithm: String,
    #[schema(default = "[\"fabric\"]")]
    pub loaders: Vec<String>,
    #[schema(default = "[\"1.16.5\"]")]
    pub game_versions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectIds {
    pub ids: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AlgorithmItems {
    #[schema(default = "[\"sha512\"]")]
    pub algorithm: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ProjectVersionQuery {
    pub game_versions: Option<String>,
    pub loaders: Option<String>,
    pub featured: Option<bool>,
}
