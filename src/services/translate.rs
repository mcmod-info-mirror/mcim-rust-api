use sqlx::PgPool;

use crate::db::translate::{get_curseforge_translations, get_modrinth_translations};
use crate::errors::ServiceError;
use crate::models::translate::responses::{
    CurseForgeTranslationResponse, ModrinthTranslationResponse,
};

pub struct ModrinthService {
    pub db: PgPool,
}

impl ModrinthService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_translation(
        &self,
        project_id: &str,
    ) -> Result<ModrinthTranslationResponse, ServiceError> {
        if project_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("project_id"),
                reason: String::from("Project ID cannot be empty"),
            });
        }

        let translation = get_modrinth_translations(&self.db, &[project_id.to_string()])
            .await?
            .pop();
        match translation {
            Some(doc) => {
                // translated is null
                if doc.translated.is_none() {
                    return Err(ServiceError::NotFound {
                        resource: String::from("Modrinth translation"),
                        detail: Some(format!(
                            "Translation for Project ID {} is not available",
                            project_id
                        )),
                    });
                }
                Ok(doc.into())
            }
            None => Err(ServiceError::NotFound {
                resource: String::from("Modrinth translation"),
                detail: Some(format!("Project ID {}", project_id)),
            }),
        }
    }

    pub async fn get_translations_batch(
        &self,
        project_ids: Vec<String>,
    ) -> Result<Vec<ModrinthTranslationResponse>, ServiceError> {
        if project_ids.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("project_ids"),
                reason: String::from("Project IDs cannot be empty"),
            });
        }

        Ok(get_modrinth_translations(&self.db, &project_ids)
            .await?
            .into_iter()
            .filter(|doc| doc.translated.is_some()) // Skip entries without translations
            .map(Into::into)
            .collect())
    }
}

pub struct CurseForgeService {
    pub db: PgPool,
}

impl CurseForgeService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_translation(
        &self,
        mod_id: i32,
    ) -> Result<CurseForgeTranslationResponse, ServiceError> {
        if mod_id <= 0 {
            return Err(ServiceError::InvalidInput {
                field: String::from("mod_id"),
                reason: String::from("Mod ID must be a positive integer"),
            });
        }

        let translation = get_curseforge_translations(&self.db, &[mod_id])
            .await?
            .pop();
        match translation {
            Some(doc) => {
                if doc.translated.is_none() {
                    return Err(ServiceError::NotFound {
                        resource: String::from("CurseForge translation"),
                        detail: Some(format!(
                            "Translation for Mod ID {} is not available",
                            mod_id
                        )),
                    });
                }
                Ok(doc.into())
            }
            None => Err(ServiceError::NotFound {
                resource: String::from("CurseForge translation"),
                detail: Some(format!("Mod ID {}", mod_id)),
            }),
        }
    }

    pub async fn get_translations_batch(
        &self,
        mod_ids: Vec<i32>,
    ) -> Result<Vec<CurseForgeTranslationResponse>, ServiceError> {
        if mod_ids.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: String::from("mod_ids"),
                reason: String::from("Mod IDs cannot be empty"),
            });
        }

        Ok(get_curseforge_translations(&self.db, &mod_ids)
            .await?
            .into_iter()
            .filter(|doc| doc.translated.is_some()) // Skip entries without translations
            .map(Into::into)
            .collect())
    }
}
