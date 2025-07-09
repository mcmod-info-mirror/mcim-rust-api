use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ModrinthTranslationResponse {
    pub project_id: String,
    pub translated: String,
    pub original: String,
    pub translated_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct CurseForgeTranslationResponse {
    pub modid: i32,
    pub translated: String,
    pub original: String,
    pub translated_at: String,
}
