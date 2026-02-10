use std::collections::HashMap;

use crate::db::database::get_database_name;
use crate::errors::ServiceError;
use crate::models::common::responses::StatisticsResponse;
use mongodb::bson::doc;

async fn get_collection_count(
    db: &mongodb::Client,
    collection_name: &str,
) -> Result<u64, ServiceError> {
    let collection = db
        .database(get_database_name().as_str())
        .collection::<mongodb::bson::Document>(collection_name);

    // 如果 $collStats 失败，回退到 estimated_document_count
    let count =
        collection
            .estimated_document_count()
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!(
                    "Failed to get collection count for {}: {}",
                    collection_name, e
                ),
                source: Some(e),
            })?;
    Ok(count)
}

async fn get_translated_count(
    db: &mongodb::Client,
    collection_name: &str,
) -> Result<u64, ServiceError> {
    let collection = db
        .database(get_database_name().as_str())
        .collection::<mongodb::bson::Document>(collection_name);

    // 只计算 translated_at 不为 null 的文档
    let filter = doc! { "translated_at": { "$ne": null } };
    let count =
        collection
            .count_documents(filter)
            .await
            .map_err(|e| ServiceError::DatabaseError {
                message: format!(
                    "Failed to get translated count for {}: {}",
                    collection_name, e
                ),
                source: Some(e),
            })?;
    Ok(count)
}

pub async fn get_statistics_info(
    modrinth: bool,
    curseforge: bool,
    translate: bool,
    db: &mongodb::Client,
) -> Result<StatisticsResponse, ServiceError> {
    let mut modrinth_statistics = HashMap::new();
    if modrinth {
        let modrinth_projects_count = get_collection_count(db, "modrinth_projects").await?;
        modrinth_statistics.insert("project".to_string(), modrinth_projects_count);

        let modrinth_versions_count = get_collection_count(db, "modrinth_versions").await?;
        modrinth_statistics.insert("version".to_string(), modrinth_versions_count);

        let modrinth_files_count = get_collection_count(db, "modrinth_files").await?;
        modrinth_statistics.insert("file".to_string(), modrinth_files_count);
    }

    let mut curseforge_statistics = HashMap::new();
    if curseforge {
        let curseforge_mods_count = get_collection_count(db, "curseforge_mods").await?;
        curseforge_statistics.insert("mod".to_string(), curseforge_mods_count);

        let curseforge_files_count = get_collection_count(db, "curseforge_files").await?;
        curseforge_statistics.insert("file".to_string(), curseforge_files_count);

        // let curseforge_fingerprints_count =
        //     get_collection_count(db, "curseforge_fingerprints").await?;
        // curseforge_statistics.insert("fingerprint".to_string(), curseforge_fingerprints_count);
    }

    let mut translate_statistics = HashMap::new();
    if translate {
        let curseforge_translate_count = get_translated_count(db, "curseforge_translated").await?;
        translate_statistics.insert("curseforge".to_string(), curseforge_translate_count);

        let modrinth_translate_count = get_translated_count(db, "modrinth_translated").await?;
        translate_statistics.insert("modrinth".to_string(), modrinth_translate_count);
    }

    Ok(StatisticsResponse {
        curseforge: Some(curseforge_statistics),
        modrinth: Some(modrinth_statistics),
        translate: Some(translate_statistics),
    })
}
