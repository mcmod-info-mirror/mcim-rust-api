use serde::Serialize;

#[derive(Serialize)]
pub struct ModrinthTranslationResponse {
    pub project_id: String,
    pub translated: String,
    pub original: String,
    pub translated_at: String,
}

#[derive(Serialize)]
pub struct CurseForgeTranslationResponse {
    pub modid: i32,
    pub translated: String,
    pub original: String,
    pub translated_at: String,
}
