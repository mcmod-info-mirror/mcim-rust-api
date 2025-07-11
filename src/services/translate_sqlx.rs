use crate::errors::ServiceError;
use crate::models::translate::entities::{CurseForgeTranslation, ModrinthTranslation};
use sqlx::PgPool;

pub struct ModrinthService {
    pub pgpool: PgPool,
}

impl ModrinthService {
    pub fn new(pgpool: PgPool) -> Self {
        Self { pgpool }
    }

    pub async fn get_translation(
        &self,
        project_id: &str,
    ) -> Result<Option<ModrinthTranslation>, ServiceError> {
        if project_id.trim().is_empty() {
            return Err(ServiceError::InvalidInput {
                field: "project_id".to_string(),
                reason: "Project ID cannot be empty".to_string(),
            });
        }

        let rec = sqlx::query_as::<_, ModrinthTranslation>(
            r#"SELECT project_id, translated, original, translated_at FROM modrinth_translation WHERE project_id = $1"#,
        )
        .bind(project_id)
        .fetch_optional(&self.pgpool)
        .await
        .map_err(|e| ServiceError::SqlxError {
            message: e.to_string(),
            source: e.into(),
        })?;
        Ok(rec)
    }
    pub async fn get_translations_batch(
        &self,
        project_ids: Vec<String>,
    ) -> Result<Vec<ModrinthTranslation>, ServiceError> {
        if project_ids.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: "project_ids".to_string(),
                reason: "Project IDs cannot be empty".to_string(),
            });
        }
        let recs = sqlx::query_as::<_, ModrinthTranslation>(
            r#"SELECT project_id, translated, original, translated_at FROM modrinth_translation WHERE project_id = ANY($1)"#
        )
        .bind(&project_ids)
        .fetch_all(&self.pgpool)
        .await.map_err(|e| ServiceError::SqlxError {
            message: e.to_string(),
            source: e.into(),
                })?;
        Ok(recs)
    }
}

pub struct CurseForgeService {
    pub pgpool: PgPool,
}

impl CurseForgeService {
    pub fn new(pgpool: PgPool) -> Self {
        Self { pgpool }
    }

    pub async fn get_translation(
        &self,
        mod_id: i32,
    ) -> Result<Option<CurseForgeTranslation>, ServiceError> {
        if mod_id <= 0 {
            return Err(ServiceError::InvalidInput {
                field: "mod_id".to_string(),
                reason: "Mod ID must be a positive integer".to_string(),
            });
        }

        let rec = sqlx::query_as::<_, CurseForgeTranslation>(
            r#"SELECT mod_id, translated, original, translated_at FROM curseforge_translation WHERE mod_id = $1"#,
        )
        .bind(mod_id)
        .fetch_optional(&self.pgpool)
        .await.map_err(|e| ServiceError::SqlxError {
            message: e.to_string(),
            source: e.into(),
        })?;
        Ok(rec)
    }

    pub async fn get_translations_batch(
        &self,
        mod_ids: Vec<i32>,
    ) -> Result<Vec<CurseForgeTranslation>, ServiceError> {
        if mod_ids.is_empty() {
            return Err(ServiceError::InvalidInput {
                field: "mod_ids".to_string(),
                reason: "Mod IDs cannot be empty".to_string(),
            });
        }

        let recs = sqlx::query_as::<_, CurseForgeTranslation>(
            r#"SELECT mod_id, translated, original, translated_at FROM curseforge_translation WHERE mod_id = ANY($1)"#
        )
        .bind(&mod_ids)
        .fetch_all(&self.pgpool)
        .await.map_err(|e| ServiceError::SqlxError {
            message: e.to_string(),
            source: e.into(),
        })?;
        Ok(recs)
    }
}
