use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::models::translate::entities::{CurseForgeTranslation, ModrinthTranslation};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ModrinthTranslationResponse {
    pub project_id: String,
    pub translated: String,
    pub original: String,
    pub translated_at: DateTime<Utc>,
}

impl From<ModrinthTranslation> for ModrinthTranslationResponse {
    fn from(translation: ModrinthTranslation) -> Self {
        Self {
            project_id: translation.project_id,
            translated: translation.translated.unwrap_or_default(),
            original: translation.original.unwrap_or_default(),
            translated_at: translation.translated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CurseForgeTranslationResponse {
    pub modid: i32,
    pub translated: String,
    pub original: String,
    pub translated_at: DateTime<Utc>,
}

impl From<CurseForgeTranslation> for CurseForgeTranslationResponse {
    fn from(translation: CurseForgeTranslation) -> Self {
        Self {
            modid: translation.mod_id,
            translated: translation.translated.unwrap_or_default(),
            original: translation.original.unwrap_or_default(),
            translated_at: translation.translated_at,
        }
    }
}
