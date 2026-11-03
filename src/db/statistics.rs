//! `/statistics` 用到的计数查询。

use sqlx::PgPool;

/// 参与统计的主表
#[derive(Debug, Clone, Copy)]
pub enum CountedTable {
    ModrinthProjects,
    ModrinthVersions,
    ModrinthFiles,
    CurseforgeMods,
    CurseforgeFiles,
}

impl CountedTable {
    fn name(self) -> &'static str {
        match self {
            Self::ModrinthProjects => "modrinth_projects",
            Self::ModrinthVersions => "modrinth_versions",
            Self::ModrinthFiles => "modrinth_files",
            Self::CurseforgeMods => "curseforge_mods",
            Self::CurseforgeFiles => "curseforge_files",
        }
    }

    fn exact_count_sql(self) -> &'static str {
        match self {
            Self::ModrinthProjects => "SELECT count(*) FROM modrinth_projects",
            Self::ModrinthVersions => "SELECT count(*) FROM modrinth_versions",
            Self::ModrinthFiles => "SELECT count(*) FROM modrinth_files",
            Self::CurseforgeMods => "SELECT count(*) FROM curseforge_mods",
            Self::CurseforgeFiles => "SELECT count(*) FROM curseforge_files",
        }
    }
}

/// 估算行数（pg_class.reltuples，对应 MongoDB estimated_document_count）；
/// 表从未被 ANALYZE 过时回退到精确 count(*)
pub async fn estimated_count(pool: &PgPool, table: CountedTable) -> Result<u64, sqlx::Error> {
    let estimate: Option<i64> = sqlx::query_scalar(
        "SELECT CASE WHEN reltuples >= 0 THEN reltuples::bigint END \
         FROM pg_class WHERE oid = $1::regclass",
    )
    .bind(table.name())
    .fetch_one(pool)
    .await?;
    let count = match estimate {
        Some(count) => count,
        None => {
            sqlx::query_scalar(table.exact_count_sql())
                .fetch_one(pool)
                .await?
        }
    };
    Ok(count.max(0) as u64)
}

/// 已有译文（translated_at 非空）的 Modrinth 翻译条数
pub async fn modrinth_translated_count(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM modrinth_translated WHERE translated_at IS NOT NULL",
    )
    .fetch_one(pool)
    .await?;
    Ok(count as u64)
}

/// 已有译文（translated_at 非空）的 CurseForge 翻译条数
pub async fn curseforge_translated_count(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM curseforge_translated WHERE translated_at IS NOT NULL",
    )
    .fetch_one(pool)
    .await?;
    Ok(count as u64)
}
