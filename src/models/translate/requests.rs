use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct CurseforgeQuery {
    pub mod_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthQuery {
    pub project_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModrinthTranslationRequest {
    pub project_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurseForgeTranslationRequest {
    pub modids: Vec<i32>,
}