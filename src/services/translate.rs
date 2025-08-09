use bson::doc;
use futures::stream::StreamExt;
use mongodb::Client;

use crate::db::database::get_database_name;
use crate::errors::ServiceError;
use crate::models::translate::entities::{CurseForgeTranslation, ModrinthTranslation};
use crate::models::translate::responses::{
    CurseForgeTranslationResponse, ModrinthTranslationResponse,
};

pub struct ModrinthService {
    pub db: Client,
}

impl ModrinthService {
    pub fn new(db: Client) -> Self {
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

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<ModrinthTranslation>("modrinth_translated");

        match collection.find_one(doc! { "_id": project_id }).await? {
            Some(doc) => Ok(doc.into()),
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

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<ModrinthTranslation>("modrinth_translated");

        let filter = doc! { "_id": { "$in": project_ids } };
        let mut cursor = collection.find(filter).await?;
        let mut results = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(doc) => {
                    results.push(doc.into());
                }
                Err(e) => {
                    return Err(ServiceError::DatabaseError {
                        message: e.to_string(),
                        source: Some(e),
                    })
                }
            }
        }

        Ok(results)
    }
}

pub struct CurseForgeService {
    pub db: Client,
}

impl CurseForgeService {
    pub fn new(db: Client) -> Self {
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

        let collection = self
            .db
            .database(&get_database_name().as_str())
            .collection::<CurseForgeTranslation>("curseforge_translated");

        match collection.find_one(doc! { "_id": mod_id }).await? {
            Some(doc) => Ok(doc.into()),
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

        let collection = self
            .db
            .database(&get_database_name().as_str())
            .collection::<CurseForgeTranslation>("curseforge_translated");

        let filter = doc! { "_id": { "$in": mod_ids } };
        let mut cursor = collection.find(filter).await?;
        let mut results = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(doc) => {
                    results.push(doc.into());
                }
                Err(e) => {
                    return Err(ServiceError::DatabaseError {
                        message: e.to_string(),
                        source: Some(e),
                    })
                }
            }
        }

        Ok(results)
    }
}
