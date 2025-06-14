use crate::config::database::get_database_name;

use crate::errors::ServiceError;

async fn get_collection_count(db: &mongodb::Client, collection_name: &str) -> Result<u64, ServiceError> {
    let collection = db.database(get_database_name().as_str()).collection::<mongodb::bson::Document>(collection_name);

    // 如果 $collStats 失败，回退到 estimated_document_count
    let count = collection.estimated_document_count(None).await.map_err(|e| {
            ServiceError::Database {
                message: format!("Failed to get collection count for {}: {}", collection_name, e), source: Some(e.into()),
            }
    })?;
    Ok(count)
}

pub async fn get_statistics_info(modrinth: bool, curseforge: bool, translate: bool, db: &mongodb::Client) -> Result<serde_json::Value, ServiceError> {
    let mut statistics = serde_json::Map::new();
    if modrinth {
        let modrinth_projects_count = get_collection_count(db, "modrinth_projects").await?;
        statistics.insert("modrinth_projects".to_string(), serde_json::Value::from(modrinth_projects_count));

        let modrinth_versions_count = get_collection_count(db, "modrinth_versions").await?;
        statistics.insert("modrinth_versions".to_string(), serde_json::Value::from(modrinth_versions_count));

        let modrinth_files_count = get_collection_count(db, "modrinth_files").await?;
        statistics.insert("modrinth_files".to_string(), serde_json::Value::from(modrinth_files_count));
    }

    if curseforge {
        let curseforge_mods_count = get_collection_count(db, "curseforge_mods").await?;
        statistics.insert("curseforge_mods".to_string(), serde_json::Value::from(curseforge_mods_count));

        let curseforge_files_count = get_collection_count(db, "curseforge_files").await?;
        statistics.insert("curseforge_files".to_string(), serde_json::Value::from(curseforge_files_count));

        let curseforge_fingerprints_count = get_collection_count(db, "curseforge_fingerprints").await?;
        statistics.insert("curseforge_fingerprints".to_string(), serde_json::Value::from(curseforge_fingerprints_count));
    }

    if translate {
        let curseforge_translate_count = get_collection_count(db, "curseforge_translated").await?;
        statistics.insert("curseforge_translated".to_string(), serde_json::Value::from(curseforge_translate_count));

        let modrinth_translate_count = get_collection_count(db, "modrinth_translated").await?;
        statistics.insert("modrinth_translated".to_string(), serde_json::Value::from(modrinth_translate_count));
    }

    Ok(serde_json::Value::Object(statistics))
}