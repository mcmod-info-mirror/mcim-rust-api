use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CurseforgeQuery {
    #[serde(rename = "modId")]
    pub mod_id: i32,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ModrinthQuery {
    pub project_id: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ModrinthTranslationRequest {
    pub project_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CurseForgeTranslationRequest {
    pub modids: Vec<i32>,
}
