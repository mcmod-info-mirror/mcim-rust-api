use bson::doc;
use futures::stream::StreamExt;
use mongodb::Client;

use crate::config::database::get_database_name;
use crate::models::translate::entities::{CurseForgeTranslation, ModrinthTranslation};
use crate::errors::ServiceError;

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
    ) -> Result<Option<ModrinthTranslation>, ServiceError> {
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

        match collection
            .find_one(doc! { "_id": project_id }, None)
            .await?
        {
            Some(doc) => {
                Ok(Some(doc))
            }
            None => Ok(None),
        }
    }

    pub async fn get_translations_batch(
        &self,
        project_ids: Vec<String>,
    ) -> Result<Vec<ModrinthTranslation>, ServiceError> {
        if project_ids.is_empty() {
            return Err(ServiceError::InvalidInput  {
                field: String::from("project_ids"),
                reason: String::from("Project IDs cannot be empty"),
            });
        }

        let collection = self
            .db
            .database(get_database_name().as_str())
            .collection::<ModrinthTranslation>("modrinth_translated");

        let filter = doc! { "_id": { "$in": project_ids } };
        let mut cursor = collection.find(filter, None).await?;
        let mut results = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(doc) => {
                    results.push(doc);
                }
                Err(e) => return Err(ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                }),
            }
        }

        Ok(results)
    }

    // fn convert_document_to_response(
    //     &self,
    //     doc: Document,
    // ) -> Result<ModrinthTranslationResponse, ServiceError> {
    //     let translated = doc
    //         .get_str("translated")
    //         .map_err(|_| ServiceError::UnexpectedError(String::from("Missing translated field")))?
    //         .to_string();

    //     let original = doc
    //         .get_str("original")
    //         .map_err(|_| ServiceError::UnexpectedError(String::from("Missing original field")))?
    //         .to_string();

    //     let translated_at = match doc.get_datetime("translated_at") {
    //         Ok(bson_dt) => {
    //             let chrono_dt = bson_dt.to_chrono();
    //             chrono_dt.format("%Y-%m-%d %H:%M:%S").to_string()
    //         }
    //         Err(_) => return Err(ServiceError::UnexpectedError(String::from("Invalid translated_at field"))),
    //     };

    //     let project_id = doc
    //         .get_str("_id")
    //         .map_err(|_| ServiceError::UnexpectedError(String::from("Missing project_id field")))?;

    //     Ok(ModrinthTranslationResponse {
    //         project_id: project_id.to_string(),
    //         translated,
    //         original,
    //         translated_at,
    //     })
    // }
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
    ) -> Result<Option<CurseForgeTranslation>, ServiceError> {
        if mod_id <= 0 {
            return Err(ServiceError::InvalidInput { field: String::from("mod_id"), reason: String::from("Mod ID must be a positive integer") });
        }

        let collection = self
            .db
            .database(&get_database_name().as_str())
            .collection::<CurseForgeTranslation>("curseforge_translated");

        match collection.find_one(doc! { "_id": mod_id }, None).await? {
            Some(doc) => {
                Ok(Some(doc))
            }
            None => Ok(None),
        }
    }

    pub async fn get_translations_batch(
        &self,
        mod_ids: Vec<i32>,
    ) -> Result<Vec<CurseForgeTranslation>, ServiceError> {
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
        let mut cursor = collection.find(filter, None).await?;
        let mut results = Vec::new();

        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(doc) => {
                        results.push(doc);
                }
                Err(e) => return Err(ServiceError::DatabaseError {
                    message: e.to_string(),
                    source: Some(e),
                }),
            }
        }

        Ok(results)
    }

    // fn convert_document_to_response(
    //     &self,
    //     doc: Document,
    // ) -> Result<CurseForgeTranslationResponse, ServiceError> {
    //     let translated = doc
    //         .get_str("translated")
    //         .map_err(|_| ServiceError::UnexpectedError(String::from("Missing translated field")))?
    //         .to_string();

    //     let original = doc
    //         .get_str("original")
    //         .map_err(|_| ServiceError::UnexpectedError(String::from("Missing original field")))?
    //         .to_string();

    //     let translated_at = match doc.get_datetime("translated_at") {
    //         Ok(bson_dt) => {
    //             let chrono_dt = bson_dt.to_chrono();
    //             chrono_dt.format("%Y-%m-%d %H:%M:%S").to_string()
    //         }
    //         Err(_) => return Err(ServiceError::UnexpectedError(String::from("Invalid translated_at field"))),
    //     };

    //     let mod_id = doc
    //         .get_i32("_id")
    //         .map_err(|_| ServiceError::UnexpectedError(String::from("Missing mod_id field")))?;

    //     Ok(CurseForgeTranslationResponse {
    //         modid: mod_id,
    //         translated,
    //         original,
    //         translated_at,
    //     })
    // }
}
