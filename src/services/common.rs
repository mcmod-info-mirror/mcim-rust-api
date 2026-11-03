use std::collections::HashMap;

use sqlx::PgPool;

use crate::db::statistics::{
    CountedTable, curseforge_translated_count, estimated_count, modrinth_translated_count,
};
use crate::errors::ServiceError;
use crate::models::common::responses::StatisticsResponse;

async fn get_table_count(db: &PgPool, table: CountedTable) -> Result<u64, ServiceError> {
    estimated_count(db, table)
        .await
        .map_err(|e| ServiceError::DatabaseError {
            message: format!("Failed to get row count for {:?}: {}", table, e),
            source: Some(e),
        })
}

pub async fn get_statistics_info(
    modrinth: bool,
    curseforge: bool,
    translate: bool,
    db: &PgPool,
) -> Result<StatisticsResponse, ServiceError> {
    let mut modrinth_statistics = HashMap::new();
    if modrinth {
        let modrinth_projects_count = get_table_count(db, CountedTable::ModrinthProjects).await?;
        modrinth_statistics.insert("project".to_string(), modrinth_projects_count);

        let modrinth_versions_count = get_table_count(db, CountedTable::ModrinthVersions).await?;
        modrinth_statistics.insert("version".to_string(), modrinth_versions_count);

        let modrinth_files_count = get_table_count(db, CountedTable::ModrinthFiles).await?;
        modrinth_statistics.insert("file".to_string(), modrinth_files_count);
    }

    let mut curseforge_statistics = HashMap::new();
    if curseforge {
        let curseforge_mods_count = get_table_count(db, CountedTable::CurseforgeMods).await?;
        curseforge_statistics.insert("mod".to_string(), curseforge_mods_count);

        let curseforge_files_count = get_table_count(db, CountedTable::CurseforgeFiles).await?;
        curseforge_statistics.insert("file".to_string(), curseforge_files_count);
    }

    let mut translate_statistics = HashMap::new();
    if translate {
        let curseforge_translate_count = curseforge_translated_count(db).await?;
        translate_statistics.insert("curseforge".to_string(), curseforge_translate_count);

        let modrinth_translate_count = modrinth_translated_count(db).await?;
        translate_statistics.insert("modrinth".to_string(), modrinth_translate_count);
    }

    Ok(StatisticsResponse {
        curseforge: Some(curseforge_statistics),
        modrinth: Some(modrinth_statistics),
        translate: Some(translate_statistics),
    })
}
