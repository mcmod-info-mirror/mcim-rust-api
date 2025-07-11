use std::collections::HashMap;

use crate::config::mongo::get_database_name;

use crate::errors::ServiceError;

async fn get_collection_count(
    db: &mongodb::Client,
    collection_name: &str,
) -> Result<u64, ServiceError> {
    let collection = db
        .database(get_database_name().as_str())
        .collection::<mongodb::bson::Document>(collection_name);

    // 如果 $collStats 失败，回退到 estimated_document_count
    let count = collection
        .estimated_document_count(None)
        .await
        .map_err(|e| ServiceError::MongoDBError {
            message: format!(
                "Failed to get collection count for {}: {}",
                collection_name, e
            ),
            source: Some(e.into()),
        })?;
    Ok(count)
}

pub async fn get_statistics_info(
    modrinth: bool,
    curseforge: bool,
    translate: bool,
    db: &mongodb::Client,
) -> Result<serde_json::Value, ServiceError> {
    let mut statistics = HashMap::new();
    if modrinth {
        let mut modrinth_statistics = HashMap::new();
        let modrinth_projects_count = get_collection_count(db, "modrinth_projects").await?;
        modrinth_statistics.insert("project".to_string(), modrinth_projects_count);

        let modrinth_versions_count = get_collection_count(db, "modrinth_versions").await?;
        modrinth_statistics.insert("version".to_string(), modrinth_versions_count);

        let modrinth_files_count = get_collection_count(db, "modrinth_files").await?;
        modrinth_statistics.insert("file".to_string(), modrinth_files_count);

        statistics.insert("modrinth".to_string(), modrinth_statistics);
    }

    if curseforge {
        let mut curseforge_statistics = HashMap::new();
        let curseforge_mods_count = get_collection_count(db, "curseforge_mods").await?;
        curseforge_statistics.insert("mod".to_string(), curseforge_mods_count);

        let curseforge_files_count = get_collection_count(db, "curseforge_files").await?;
        curseforge_statistics.insert("file".to_string(), curseforge_files_count);

        let curseforge_fingerprints_count =
            get_collection_count(db, "curseforge_fingerprints").await?;
        curseforge_statistics.insert("fingerprint".to_string(), curseforge_fingerprints_count);
        statistics.insert("curseforge".to_string(), curseforge_statistics);
    }

    if translate {
        let mut translate_statistics = HashMap::new();
        let curseforge_translate_count = get_collection_count(db, "curseforge_translated").await?;
        translate_statistics.insert("curseforge".to_string(), curseforge_translate_count);

        let modrinth_translate_count = get_collection_count(db, "modrinth_translated").await?;
        translate_statistics.insert("modrinth".to_string(), modrinth_translate_count);
        statistics.insert("translate".to_string(), translate_statistics);
    }

    Ok(serde_json::json!(statistics))
}
