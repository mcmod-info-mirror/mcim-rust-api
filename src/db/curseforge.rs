//! CurseForge 关系表读写。表结构见 `migrations/`。

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection, PgPool};

use crate::db::support::{
    allocate_ids, fetch_grouped, fetch_string_lists, flatten_string_lists, read_snapshot,
};
use crate::models::curseforge::entities::{
    Author, Category, CategoryInfo, File, FileDependencies, FileIndex, FileInfo,
    FileSortableGameVersions, Hash, Links, Logo, Mod, Module, ScreenShot,
};

/// `curseforge_file_records` 中映射到 `FileInfo` 的列（表别名 r）
macro_rules! file_record_columns {
    () => {
        "r.id, r.game_id, r.mod_id, r.is_available, r.display_name, r.file_name, \
         r.release_type, r.file_status, r.file_date, r.file_length, r.download_count, \
         r.file_size_on_disk, r.download_url, r.expose_as_alternative, \
         r.parent_project_file_id, r.alternate_file_id, r.is_server_pack, \
         r.server_pack_file_id, r.is_early_access_content, r.early_access_end_date, \
         r.file_fingerprint"
    };
}

/// `curseforge_files` 查询的 SELECT 部分（f = curseforge_files, r = curseforge_file_records）
macro_rules! select_files {
    () => {
        concat!(
            "SELECT f.record_id, ",
            file_record_columns!(),
            ", f.sync_at, f.found, f.file_cdn_cached, f.need_to_cache, f.md5, f.sha1 \
             FROM curseforge_files f JOIN curseforge_file_records r ON r.record_id = f.record_id "
        )
    };
}

/// curseforge_files 中满足：属于 $1 这个 Mod，且 gameVersions 包含 $2 中全部值
macro_rules! mod_files_filter {
    () => {
        "WHERE r.mod_id = $1 AND NOT EXISTS ( \
             SELECT 1 FROM unnest($2::text[]) AS required(game_version) \
             WHERE NOT EXISTS ( \
                 SELECT 1 FROM curseforge_file_record_game_versions g \
                 WHERE g.record_id = r.record_id AND g.game_version = required.game_version)) "
    };
}

const FILE_RECORD_SEQUENCE: &str = "SELECT nextval(pg_get_serial_sequence('curseforge_file_records', 'record_id')) \
     FROM generate_series(1, $1)";

#[derive(FromRow)]
struct FileRow {
    record_id: i64,
    #[sqlx(flatten)]
    file: File,
}

#[derive(FromRow)]
struct LatestFileRow {
    owner_mod_id: i32,
    record_id: i64,
    #[sqlx(flatten)]
    info: FileInfo,
}

// ---------------------------------------------------------------------------
// 读取
// ---------------------------------------------------------------------------

/// 为文件记录填充数组字段（hashes、gameVersions、sortableGameVersions、dependencies、modules）
async fn fill_file_records(
    conn: &mut PgConnection,
    records: Vec<(i64, &mut FileInfo)>,
) -> Result<(), sqlx::Error> {
    let ids: Vec<i64> = records.iter().map(|(id, _)| *id).collect();

    let mut hashes: HashMap<i64, Vec<Hash>> = fetch_grouped(
        conn,
        "SELECT record_id AS parent_key, value, algo FROM curseforge_file_record_hashes \
         WHERE record_id = ANY($1) ORDER BY record_id, position",
        &ids,
    )
    .await?;
    let mut game_versions = fetch_string_lists(
        conn,
        "SELECT record_id, array_agg(game_version ORDER BY position) \
         FROM curseforge_file_record_game_versions WHERE record_id = ANY($1) GROUP BY record_id",
        &ids,
    )
    .await?;
    let mut sortable: HashMap<i64, Vec<FileSortableGameVersions>> = fetch_grouped(
        conn,
        "SELECT record_id AS parent_key, game_version_name, game_version_padded, game_version, \
         game_version_release_date, game_version_type_id \
         FROM curseforge_file_record_sortable_game_versions \
         WHERE record_id = ANY($1) ORDER BY record_id, position",
        &ids,
    )
    .await?;
    let mut dependencies: HashMap<i64, Vec<FileDependencies>> = fetch_grouped(
        conn,
        "SELECT record_id AS parent_key, mod_id, relation_type FROM curseforge_file_record_dependencies \
         WHERE record_id = ANY($1) ORDER BY record_id, position",
        &ids,
    )
    .await?;
    let mut modules: HashMap<i64, Vec<Module>> = fetch_grouped(
        conn,
        "SELECT record_id AS parent_key, name, fingerprint FROM curseforge_file_record_modules \
         WHERE record_id = ANY($1) ORDER BY record_id, position",
        &ids,
    )
    .await?;

    for (id, info) in records {
        info.hashes = Some(hashes.remove(&id).unwrap_or_default());
        info.game_versions = Some(game_versions.remove(&id).unwrap_or_default());
        info.sortable_game_versions = Some(sortable.remove(&id).unwrap_or_default());
        info.dependencies = Some(dependencies.remove(&id).unwrap_or_default());
        info.modules = Some(modules.remove(&id).unwrap_or_default());
    }
    Ok(())
}

async fn finish_files(
    conn: &mut PgConnection,
    mut rows: Vec<FileRow>,
) -> Result<Vec<File>, sqlx::Error> {
    fill_file_records(
        conn,
        rows.iter_mut()
            .map(|row| (row.record_id, &mut row.file.info))
            .collect(),
    )
    .await?;
    Ok(rows.into_iter().map(|row| row.file).collect())
}

/// 已存在的 Mod id
pub async fn existing_mod_ids(pool: &PgPool, mod_ids: &[i32]) -> Result<Vec<i32>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM curseforge_mods WHERE id = ANY($1)")
        .bind(mod_ids)
        .fetch_all(pool)
        .await
}

/// 按 Mod id 读取完整 Mod 文档，按 id 升序
pub async fn get_mods(pool: &PgPool, mod_ids: &[i32]) -> Result<Vec<Mod>, sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let conn = &mut *tx;

    let mut mods: Vec<Mod> = sqlx::query_as(
        "SELECT id, game_id, name, slug, summary, status, download_count, is_featured, \
         primary_category_id, class_id, main_file_id, date_created, date_modified, date_released, \
         allow_mod_distribution, game_popularity_rank, is_available, thumbs_up_count, rating, \
         translated_summary, sync_at, checked_at \
         FROM curseforge_mods WHERE id = ANY($1) ORDER BY id",
    )
    .bind(mod_ids)
    .fetch_all(&mut *conn)
    .await?;
    if mods.is_empty() {
        return Ok(mods);
    }
    let ids: Vec<i32> = mods.iter().map(|m| m.id).collect();

    let mut links: HashMap<i32, Vec<Links>> = fetch_grouped(
        conn,
        "SELECT mod_id AS parent_key, website_url, wiki_url, issues_url, source_url \
         FROM curseforge_mod_links WHERE mod_id = ANY($1) ORDER BY mod_id",
        &ids,
    )
    .await?;
    let mut logos: HashMap<i32, Vec<Logo>> = fetch_grouped(
        conn,
        "SELECT mod_id AS parent_key, id, logo_mod_id AS mod_id, title, description, \
         thumbnail_url, url FROM curseforge_mod_logos WHERE mod_id = ANY($1) ORDER BY mod_id",
        &ids,
    )
    .await?;
    let mut categories: HashMap<i32, Vec<CategoryInfo>> = fetch_grouped(
        conn,
        "SELECT mod_id AS parent_key, id, game_id, name, slug, url, icon_url, date_modified, \
         is_class, class_id, parent_category_id, display_index FROM curseforge_mod_categories \
         WHERE mod_id = ANY($1) ORDER BY mod_id, position",
        &ids,
    )
    .await?;
    let mut authors: HashMap<i32, Vec<Author>> = fetch_grouped(
        conn,
        "SELECT mod_id AS parent_key, id, name, url FROM curseforge_mod_authors \
         WHERE mod_id = ANY($1) ORDER BY mod_id, position",
        &ids,
    )
    .await?;
    let mut screenshots: HashMap<i32, Vec<ScreenShot>> = fetch_grouped(
        conn,
        "SELECT mod_id AS parent_key, id, screenshot_mod_id AS mod_id, title, description, \
         thumbnail_url, url FROM curseforge_mod_screenshots \
         WHERE mod_id = ANY($1) ORDER BY mod_id, position",
        &ids,
    )
    .await?;
    let mut indexes: HashMap<i32, Vec<FileIndex>> = fetch_grouped(
        conn,
        "SELECT mod_id AS parent_key, game_version, file_id, filename, release_type, \
         game_version_type_id, mod_loader FROM curseforge_mod_latest_files_indexes \
         WHERE mod_id = ANY($1) ORDER BY mod_id, position",
        &ids,
    )
    .await?;
    let mut latest_files = load_latest_files(conn, &ids).await?;

    for m in &mut mods {
        m.links = links.remove(&m.id).and_then(|v| v.into_iter().next());
        m.logo = logos.remove(&m.id).and_then(|v| v.into_iter().next());
        m.categories = Some(categories.remove(&m.id).unwrap_or_default());
        m.authors = Some(authors.remove(&m.id).unwrap_or_default());
        m.screenshots = Some(screenshots.remove(&m.id).unwrap_or_default());
        m.latest_files_indexes = Some(indexes.remove(&m.id).unwrap_or_default());
        m.latest_files = Some(latest_files.remove(&m.id).unwrap_or_default());
    }

    tx.commit().await?;
    Ok(mods)
}

async fn load_latest_files(
    conn: &mut PgConnection,
    mod_ids: &[i32],
) -> Result<HashMap<i32, Vec<FileInfo>>, sqlx::Error> {
    let mut rows: Vec<LatestFileRow> = sqlx::query_as(concat!(
        "SELECT lf.mod_id AS owner_mod_id, lf.record_id, ",
        file_record_columns!(),
        " FROM curseforge_mod_latest_files lf \
          JOIN curseforge_file_records r ON r.record_id = lf.record_id \
          WHERE lf.mod_id = ANY($1) ORDER BY lf.mod_id, lf.position"
    ))
    .bind(mod_ids)
    .fetch_all(&mut *conn)
    .await?;

    fill_file_records(
        conn,
        rows.iter_mut()
            .map(|row| (row.record_id, &mut row.info))
            .collect(),
    )
    .await?;

    let mut grouped: HashMap<i32, Vec<FileInfo>> = HashMap::new();
    for row in rows {
        grouped.entry(row.owner_mod_id).or_default().push(row.info);
    }
    Ok(grouped)
}

/// Mod id -> latestFiles；不存在的 Mod 不出现在结果中
pub async fn get_latest_files(
    pool: &PgPool,
    mod_ids: &[i32],
) -> Result<HashMap<i32, Vec<FileInfo>>, sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let existing: Vec<i32> =
        sqlx::query_scalar("SELECT id FROM curseforge_mods WHERE id = ANY($1)")
            .bind(mod_ids)
            .fetch_all(&mut *tx)
            .await?;
    let mut latest_files = load_latest_files(&mut tx, &existing).await?;
    tx.commit().await?;
    Ok(existing
        .into_iter()
        .map(|id| (id, latest_files.remove(&id).unwrap_or_default()))
        .collect())
}

/// 按文件 id 读取，按 id 升序
pub async fn get_files(pool: &PgPool, file_ids: &[i32]) -> Result<Vec<File>, sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let rows: Vec<FileRow> = sqlx::query_as(concat!(
        select_files!(),
        "WHERE f.id = ANY($1) ORDER BY f.id"
    ))
    .bind(file_ids)
    .fetch_all(&mut *tx)
    .await?;
    let files = finish_files(&mut tx, rows).await?;
    tx.commit().await?;
    Ok(files)
}

/// 按 fileFingerprint 匹配文件，可选按 gameId 过滤
pub async fn get_files_by_fingerprints(
    pool: &PgPool,
    fingerprints: &[i64],
    game_id: Option<i32>,
) -> Result<Vec<File>, sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let rows: Vec<FileRow> = sqlx::query_as(concat!(
        select_files!(),
        "WHERE r.file_fingerprint = ANY($1) AND ($2::integer IS NULL OR r.game_id = $2) \
         ORDER BY f.id"
    ))
    .bind(fingerprints)
    .bind(game_id)
    .fetch_all(&mut *tx)
    .await?;
    let files = finish_files(&mut tx, rows).await?;
    tx.commit().await?;
    Ok(files)
}

/// Mod 的文件分页：gameVersions 须包含 `required_game_versions` 全部值，按 fileDate 降序。
/// 返回 (当前页, 总数)
pub async fn get_mod_files(
    pool: &PgPool,
    mod_id: i32,
    required_game_versions: &[String],
    index: i64,
    page_size: i64,
) -> Result<(Vec<File>, i64), sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let total: i64 = sqlx::query_scalar(concat!(
        "SELECT count(*) FROM curseforge_files f \
         JOIN curseforge_file_records r ON r.record_id = f.record_id ",
        mod_files_filter!()
    ))
    .bind(mod_id)
    .bind(required_game_versions)
    .fetch_one(&mut *tx)
    .await?;
    let rows: Vec<FileRow> = sqlx::query_as(concat!(
        select_files!(),
        mod_files_filter!(),
        "ORDER BY r.file_date DESC NULLS LAST, f.id DESC OFFSET $3 LIMIT $4"
    ))
    .bind(mod_id)
    .bind(required_game_versions)
    .bind(index)
    .bind(page_size)
    .fetch_all(&mut *tx)
    .await?;
    let files = finish_files(&mut tx, rows).await?;
    tx.commit().await?;
    Ok((files, total))
}

/// 分类：指定 classId 时按 classId 过滤，否则 `classes_only` 时只返回 isClass = true 的分类
pub async fn get_categories(
    pool: &PgPool,
    game_id: i32,
    class_id: Option<i32>,
    classes_only: bool,
) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, game_id, name, slug, url, icon_url, date_modified, is_class, class_id, \
         parent_category_id, display_index, sync_at FROM curseforge_categories \
         WHERE game_id = $1 AND CASE \
             WHEN $2::integer IS NOT NULL THEN class_id = $2 \
             WHEN $3 THEN is_class \
             ELSE true END \
         ORDER BY id",
    )
    .bind(game_id)
    .bind(class_id)
    .bind(classes_only)
    .fetch_all(pool)
    .await
}

/// 文件 id -> fileLength（无 fileLength 的文件不出现在结果中）
pub async fn get_file_lengths(
    pool: &PgPool,
    file_ids: &[i64],
) -> Result<HashMap<i64, i64>, sqlx::Error> {
    let rows: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT f.id::bigint, r.file_length FROM curseforge_files f \
         JOIN curseforge_file_records r ON r.record_id = f.record_id \
         WHERE f.id = ANY($1) AND r.file_length IS NOT NULL",
    )
    .bind(file_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}

// ---------------------------------------------------------------------------
// 写入：以文档为单位整体替换（先删后插），调用方负责事务；同一批次内 id 不可重复
// ---------------------------------------------------------------------------

pub async fn upsert_categories(
    conn: &mut PgConnection,
    categories: &[Category],
) -> Result<(), sqlx::Error> {
    if categories.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO curseforge_categories (id, game_id, name, slug, url, icon_url, date_modified, \
         is_class, class_id, parent_category_id, display_index, sync_at) \
         SELECT * FROM UNNEST($1::integer[], $2::integer[], $3::text[], $4::text[], $5::text[], \
             $6::text[], $7::timestamptz[], $8::boolean[], $9::integer[], $10::integer[], \
             $11::integer[], $12::timestamptz[]) \
         ON CONFLICT (id) DO UPDATE SET game_id = EXCLUDED.game_id, name = EXCLUDED.name, \
             slug = EXCLUDED.slug, url = EXCLUDED.url, icon_url = EXCLUDED.icon_url, \
             date_modified = EXCLUDED.date_modified, is_class = EXCLUDED.is_class, \
             class_id = EXCLUDED.class_id, parent_category_id = EXCLUDED.parent_category_id, \
             display_index = EXCLUDED.display_index, sync_at = EXCLUDED.sync_at",
    )
    .bind(categories.iter().map(|c| c.id).collect::<Vec<_>>())
    .bind(categories.iter().map(|c| c.game_id).collect::<Vec<_>>())
    .bind(
        categories
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|c| c.slug.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|c| c.url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|c| c.icon_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|c| c.date_modified)
            .collect::<Vec<_>>(),
    )
    .bind(categories.iter().map(|c| c.is_class).collect::<Vec<_>>())
    .bind(categories.iter().map(|c| c.class_id).collect::<Vec<_>>())
    .bind(
        categories
            .iter()
            .map(|c| c.parent_category_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|c| c.display_index)
            .collect::<Vec<_>>(),
    )
    .bind(categories.iter().map(|c| c.sync_at).collect::<Vec<_>>())
    .execute(conn)
    .await?;
    Ok(())
}

/// 写入文件记录及其子表；`records` 中的 record_id 须由 FILE_RECORD_SEQUENCE 预分配
async fn insert_file_records(
    conn: &mut PgConnection,
    records: &[(i64, &FileInfo)],
) -> Result<(), sqlx::Error> {
    if records.is_empty() {
        return Ok(());
    }
    let ids: Vec<i64> = records.iter().map(|(id, _)| *id).collect();
    sqlx::query(
        "INSERT INTO curseforge_file_records (record_id, id, game_id, mod_id, is_available, \
         display_name, file_name, release_type, file_status, file_date, file_length, \
         download_count, file_size_on_disk, download_url, expose_as_alternative, \
         parent_project_file_id, alternate_file_id, is_server_pack, server_pack_file_id, \
         is_early_access_content, early_access_end_date, file_fingerprint) \
         SELECT * FROM UNNEST($1::bigint[], $2::integer[], $3::integer[], $4::integer[], \
             $5::boolean[], $6::text[], $7::text[], $8::integer[], $9::integer[], \
             $10::timestamptz[], $11::bigint[], $12::bigint[], $13::bigint[], $14::text[], \
             $15::boolean[], $16::integer[], $17::integer[], $18::boolean[], $19::integer[], \
             $20::boolean[], $21::timestamptz[], $22::bigint[])",
    )
    .bind(&ids)
    .bind(records.iter().map(|(_, f)| f.id).collect::<Vec<_>>())
    .bind(records.iter().map(|(_, f)| f.game_id).collect::<Vec<_>>())
    .bind(records.iter().map(|(_, f)| f.mod_id).collect::<Vec<_>>())
    .bind(
        records
            .iter()
            .map(|(_, f)| f.is_available)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.display_name.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.file_name.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.release_type)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.file_status)
            .collect::<Vec<_>>(),
    )
    .bind(records.iter().map(|(_, f)| f.file_date).collect::<Vec<_>>())
    .bind(
        records
            .iter()
            .map(|(_, f)| f.file_length)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.download_count)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.file_size_on_disk)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.download_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.expose_as_alternative)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.parent_project_file_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.alternate_file_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.is_server_pack)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.server_pack_file_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.is_early_access_content)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.early_access_end_date)
            .collect::<Vec<_>>(),
    )
    .bind(
        records
            .iter()
            .map(|(_, f)| f.file_fingerprint)
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // hashes
    let mut parents = Vec::new();
    let mut positions = Vec::new();
    let mut values = Vec::new();
    let mut algos = Vec::new();
    for (id, f) in records {
        for (i, h) in f.hashes.iter().flatten().enumerate() {
            parents.push(*id);
            positions.push(i as i32);
            values.push(h.value.as_str());
            algos.push(h.algo);
        }
    }
    sqlx::query(
        "INSERT INTO curseforge_file_record_hashes (record_id, position, value, algo) \
         SELECT * FROM UNNEST($1::bigint[], $2::integer[], $3::text[], $4::integer[])",
    )
    .bind(parents)
    .bind(positions)
    .bind(values)
    .bind(algos)
    .execute(&mut *conn)
    .await?;

    // gameVersions
    let (parents, positions, values) = flatten_string_lists(
        records
            .iter()
            .map(|(id, f)| (*id, f.game_versions.as_ref())),
    );
    sqlx::query(
        "INSERT INTO curseforge_file_record_game_versions (record_id, position, game_version) \
         SELECT * FROM UNNEST($1::bigint[], $2::integer[], $3::text[])",
    )
    .bind(parents)
    .bind(positions)
    .bind(values)
    .execute(&mut *conn)
    .await?;

    // sortableGameVersions
    let mut parents = Vec::new();
    let mut positions = Vec::new();
    let mut names = Vec::new();
    let mut paddeds = Vec::new();
    let mut versions = Vec::new();
    let mut release_dates: Vec<Option<DateTime<Utc>>> = Vec::new();
    let mut type_ids = Vec::new();
    for (id, f) in records {
        for (i, v) in f.sortable_game_versions.iter().flatten().enumerate() {
            parents.push(*id);
            positions.push(i as i32);
            names.push(v.game_version_name.as_deref());
            paddeds.push(v.game_version_padded.as_deref());
            versions.push(v.game_version.as_deref());
            release_dates.push(v.game_version_release_date);
            type_ids.push(v.game_version_type_id);
        }
    }
    sqlx::query(
        "INSERT INTO curseforge_file_record_sortable_game_versions (record_id, position, \
         game_version_name, game_version_padded, game_version, game_version_release_date, \
         game_version_type_id) \
         SELECT * FROM UNNEST($1::bigint[], $2::integer[], $3::text[], $4::text[], $5::text[], \
             $6::timestamptz[], $7::integer[])",
    )
    .bind(parents)
    .bind(positions)
    .bind(names)
    .bind(paddeds)
    .bind(versions)
    .bind(release_dates)
    .bind(type_ids)
    .execute(&mut *conn)
    .await?;

    // dependencies
    let mut parents = Vec::new();
    let mut positions = Vec::new();
    let mut mod_ids = Vec::new();
    let mut relation_types = Vec::new();
    for (id, f) in records {
        for (i, d) in f.dependencies.iter().flatten().enumerate() {
            parents.push(*id);
            positions.push(i as i32);
            mod_ids.push(d.mod_id);
            relation_types.push(d.relation_type);
        }
    }
    sqlx::query(
        "INSERT INTO curseforge_file_record_dependencies (record_id, position, mod_id, relation_type) \
         SELECT * FROM UNNEST($1::bigint[], $2::integer[], $3::integer[], $4::integer[])",
    )
    .bind(parents)
    .bind(positions)
    .bind(mod_ids)
    .bind(relation_types)
    .execute(&mut *conn)
    .await?;

    // modules
    let mut parents = Vec::new();
    let mut positions = Vec::new();
    let mut names = Vec::new();
    let mut fingerprints = Vec::new();
    for (id, f) in records {
        for (i, m) in f.modules.iter().flatten().enumerate() {
            parents.push(*id);
            positions.push(i as i32);
            names.push(m.name.as_deref());
            fingerprints.push(m.fingerprint);
        }
    }
    sqlx::query(
        "INSERT INTO curseforge_file_record_modules (record_id, position, name, fingerprint) \
         SELECT * FROM UNNEST($1::bigint[], $2::integer[], $3::text[], $4::bigint[])",
    )
    .bind(parents)
    .bind(positions)
    .bind(names)
    .bind(fingerprints)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// 整体替换 curseforge_files 文档
pub async fn replace_files(conn: &mut PgConnection, files: &[File]) -> Result<(), sqlx::Error> {
    if files.is_empty() {
        return Ok(());
    }
    let ids: Vec<i32> = files.iter().map(|f| f.info.id).collect();
    // 删除文件记录即级联删除 curseforge_files 行与全部子表行
    sqlx::query(
        "DELETE FROM curseforge_file_records WHERE record_id IN \
         (SELECT record_id FROM curseforge_files WHERE id = ANY($1))",
    )
    .bind(&ids)
    .execute(&mut *conn)
    .await?;

    let record_ids = allocate_ids(conn, FILE_RECORD_SEQUENCE, files.len()).await?;
    let records: Vec<(i64, &FileInfo)> = record_ids
        .iter()
        .copied()
        .zip(files.iter().map(|f| &f.info))
        .collect();
    insert_file_records(conn, &records).await?;

    sqlx::query(
        "INSERT INTO curseforge_files (id, record_id, sync_at, found, file_cdn_cached, \
         need_to_cache, md5, sha1) \
         SELECT * FROM UNNEST($1::integer[], $2::bigint[], $3::timestamptz[], $4::boolean[], \
             $5::boolean[], $6::boolean[], $7::text[], $8::text[])",
    )
    .bind(&ids)
    .bind(&record_ids)
    .bind(files.iter().map(|f| f.sync_at).collect::<Vec<_>>())
    .bind(files.iter().map(|f| f.found).collect::<Vec<_>>())
    .bind(files.iter().map(|f| f.file_cdn_cached).collect::<Vec<_>>())
    .bind(files.iter().map(|f| f.need_to_cache).collect::<Vec<_>>())
    .bind(files.iter().map(|f| f.md5.as_deref()).collect::<Vec<_>>())
    .bind(files.iter().map(|f| f.sha1.as_deref()).collect::<Vec<_>>())
    .execute(conn)
    .await?;
    Ok(())
}

/// 整体替换 curseforge_mods 文档（含 latestFiles 等全部嵌套结构）
pub async fn replace_mods(conn: &mut PgConnection, mods: &[Mod]) -> Result<(), sqlx::Error> {
    if mods.is_empty() {
        return Ok(());
    }
    let ids: Vec<i32> = mods.iter().map(|m| m.id).collect();
    sqlx::query(
        "DELETE FROM curseforge_file_records WHERE record_id IN \
         (SELECT record_id FROM curseforge_mod_latest_files WHERE mod_id = ANY($1))",
    )
    .bind(&ids)
    .execute(&mut *conn)
    .await?;
    sqlx::query("DELETE FROM curseforge_mods WHERE id = ANY($1)")
        .bind(&ids)
        .execute(&mut *conn)
        .await?;

    sqlx::query(
        "INSERT INTO curseforge_mods (id, game_id, name, slug, summary, status, download_count, \
         is_featured, primary_category_id, class_id, main_file_id, date_created, date_modified, \
         date_released, allow_mod_distribution, game_popularity_rank, is_available, \
         thumbs_up_count, rating, translated_summary, sync_at, checked_at) \
         SELECT * FROM UNNEST($1::integer[], $2::integer[], $3::text[], $4::text[], $5::text[], \
             $6::integer[], $7::bigint[], $8::boolean[], $9::integer[], $10::integer[], \
             $11::integer[], $12::timestamptz[], $13::timestamptz[], $14::timestamptz[], \
             $15::boolean[], $16::integer[], $17::boolean[], $18::integer[], $19::integer[], \
             $20::text[], $21::timestamptz[], $22::timestamptz[])",
    )
    .bind(&ids)
    .bind(mods.iter().map(|m| m.game_id).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.name.as_deref()).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.slug.as_str()).collect::<Vec<_>>())
    .bind(
        mods.iter()
            .map(|m| m.summary.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(mods.iter().map(|m| m.status).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.download_count).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.is_featured).collect::<Vec<_>>())
    .bind(
        mods.iter()
            .map(|m| m.primary_category_id)
            .collect::<Vec<_>>(),
    )
    .bind(mods.iter().map(|m| m.class_id).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.main_file_id).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.date_created).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.date_modified).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.date_released).collect::<Vec<_>>())
    .bind(
        mods.iter()
            .map(|m| m.allow_mod_distribution)
            .collect::<Vec<_>>(),
    )
    .bind(
        mods.iter()
            .map(|m| m.game_popularity_rank)
            .collect::<Vec<_>>(),
    )
    .bind(mods.iter().map(|m| m.is_available).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.thumbs_up_count).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.rating).collect::<Vec<_>>())
    .bind(
        mods.iter()
            .map(|m| m.translated_summary.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(mods.iter().map(|m| m.sync_at).collect::<Vec<_>>())
    .bind(mods.iter().map(|m| m.checked_at).collect::<Vec<_>>())
    .execute(&mut *conn)
    .await?;

    // links
    let with_links: Vec<(i32, &Links)> = mods
        .iter()
        .filter_map(|m| Some((m.id, m.links.as_ref()?)))
        .collect();
    sqlx::query(
        "INSERT INTO curseforge_mod_links (mod_id, website_url, wiki_url, issues_url, source_url) \
         SELECT * FROM UNNEST($1::integer[], $2::text[], $3::text[], $4::text[], $5::text[])",
    )
    .bind(with_links.iter().map(|(id, _)| *id).collect::<Vec<_>>())
    .bind(
        with_links
            .iter()
            .map(|(_, l)| l.website_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        with_links
            .iter()
            .map(|(_, l)| l.wiki_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        with_links
            .iter()
            .map(|(_, l)| l.issues_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        with_links
            .iter()
            .map(|(_, l)| l.source_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // logo
    let with_logo: Vec<(i32, &Logo)> = mods
        .iter()
        .filter_map(|m| Some((m.id, m.logo.as_ref()?)))
        .collect();
    sqlx::query(
        "INSERT INTO curseforge_mod_logos (mod_id, id, logo_mod_id, title, description, \
         thumbnail_url, url) \
         SELECT * FROM UNNEST($1::integer[], $2::integer[], $3::integer[], $4::text[], \
             $5::text[], $6::text[], $7::text[])",
    )
    .bind(with_logo.iter().map(|(id, _)| *id).collect::<Vec<_>>())
    .bind(with_logo.iter().map(|(_, l)| l.id).collect::<Vec<_>>())
    .bind(with_logo.iter().map(|(_, l)| l.mod_id).collect::<Vec<_>>())
    .bind(
        with_logo
            .iter()
            .map(|(_, l)| l.title.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        with_logo
            .iter()
            .map(|(_, l)| l.description.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        with_logo
            .iter()
            .map(|(_, l)| l.thumbnail_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        with_logo
            .iter()
            .map(|(_, l)| l.url.as_deref())
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // categories
    let categories: Vec<(i32, i32, &CategoryInfo)> = mods
        .iter()
        .flat_map(|m| {
            m.categories
                .iter()
                .flatten()
                .enumerate()
                .map(move |(i, c)| (m.id, i as i32, c))
        })
        .collect();
    sqlx::query(
        "INSERT INTO curseforge_mod_categories (mod_id, position, id, game_id, name, slug, url, \
         icon_url, date_modified, is_class, class_id, parent_category_id, display_index) \
         SELECT * FROM UNNEST($1::integer[], $2::integer[], $3::integer[], $4::integer[], \
             $5::text[], $6::text[], $7::text[], $8::text[], $9::timestamptz[], $10::boolean[], \
             $11::integer[], $12::integer[], $13::integer[])",
    )
    .bind(categories.iter().map(|(m, _, _)| *m).collect::<Vec<_>>())
    .bind(categories.iter().map(|(_, p, _)| *p).collect::<Vec<_>>())
    .bind(categories.iter().map(|(_, _, c)| c.id).collect::<Vec<_>>())
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.game_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.name.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.slug.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.icon_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.date_modified)
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.is_class)
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.class_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.parent_category_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        categories
            .iter()
            .map(|(_, _, c)| c.display_index)
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // authors
    let authors: Vec<(i32, i32, &Author)> = mods
        .iter()
        .flat_map(|m| {
            m.authors
                .iter()
                .flatten()
                .enumerate()
                .map(move |(i, a)| (m.id, i as i32, a))
        })
        .collect();
    sqlx::query(
        "INSERT INTO curseforge_mod_authors (mod_id, position, id, name, url) \
         SELECT * FROM UNNEST($1::integer[], $2::integer[], $3::integer[], $4::text[], $5::text[])",
    )
    .bind(authors.iter().map(|(m, _, _)| *m).collect::<Vec<_>>())
    .bind(authors.iter().map(|(_, p, _)| *p).collect::<Vec<_>>())
    .bind(authors.iter().map(|(_, _, a)| a.id).collect::<Vec<_>>())
    .bind(
        authors
            .iter()
            .map(|(_, _, a)| a.name.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        authors
            .iter()
            .map(|(_, _, a)| a.url.as_deref())
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // screenshots
    let screenshots: Vec<(i32, i32, &ScreenShot)> = mods
        .iter()
        .flat_map(|m| {
            m.screenshots
                .iter()
                .flatten()
                .enumerate()
                .map(move |(i, s)| (m.id, i as i32, s))
        })
        .collect();
    sqlx::query(
        "INSERT INTO curseforge_mod_screenshots (mod_id, position, id, screenshot_mod_id, title, \
         description, thumbnail_url, url) \
         SELECT * FROM UNNEST($1::integer[], $2::integer[], $3::integer[], $4::integer[], \
             $5::text[], $6::text[], $7::text[], $8::text[])",
    )
    .bind(screenshots.iter().map(|(m, _, _)| *m).collect::<Vec<_>>())
    .bind(screenshots.iter().map(|(_, p, _)| *p).collect::<Vec<_>>())
    .bind(screenshots.iter().map(|(_, _, s)| s.id).collect::<Vec<_>>())
    .bind(
        screenshots
            .iter()
            .map(|(_, _, s)| s.mod_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        screenshots
            .iter()
            .map(|(_, _, s)| s.title.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        screenshots
            .iter()
            .map(|(_, _, s)| s.description.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        screenshots
            .iter()
            .map(|(_, _, s)| s.thumbnail_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        screenshots
            .iter()
            .map(|(_, _, s)| s.url.as_deref())
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // latestFilesIndexes
    let indexes: Vec<(i32, i32, &FileIndex)> = mods
        .iter()
        .flat_map(|m| {
            m.latest_files_indexes
                .iter()
                .flatten()
                .enumerate()
                .map(move |(i, x)| (m.id, i as i32, x))
        })
        .collect();
    sqlx::query(
        "INSERT INTO curseforge_mod_latest_files_indexes (mod_id, position, game_version, file_id, \
         filename, release_type, game_version_type_id, mod_loader) \
         SELECT * FROM UNNEST($1::integer[], $2::integer[], $3::text[], $4::integer[], \
             $5::text[], $6::integer[], $7::integer[], $8::integer[])",
    )
    .bind(indexes.iter().map(|(m, _, _)| *m).collect::<Vec<_>>())
    .bind(indexes.iter().map(|(_, p, _)| *p).collect::<Vec<_>>())
    .bind(
        indexes
            .iter()
            .map(|(_, _, x)| x.game_version.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        indexes
            .iter()
            .map(|(_, _, x)| x.file_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        indexes
            .iter()
            .map(|(_, _, x)| x.filename.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        indexes
            .iter()
            .map(|(_, _, x)| x.release_type)
            .collect::<Vec<_>>(),
    )
    .bind(
        indexes
            .iter()
            .map(|(_, _, x)| x.game_version_type_id)
            .collect::<Vec<_>>(),
    )
    .bind(
        indexes
            .iter()
            .map(|(_, _, x)| x.mod_loader)
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // latestFiles
    let latest: Vec<(i32, i32, &FileInfo)> = mods
        .iter()
        .flat_map(|m| {
            m.latest_files
                .iter()
                .flatten()
                .enumerate()
                .map(move |(i, f)| (m.id, i as i32, f))
        })
        .collect();
    let record_ids = allocate_ids(conn, FILE_RECORD_SEQUENCE, latest.len()).await?;
    let records: Vec<(i64, &FileInfo)> = record_ids
        .iter()
        .copied()
        .zip(latest.iter().map(|(_, _, f)| *f))
        .collect();
    insert_file_records(conn, &records).await?;
    sqlx::query(
        "INSERT INTO curseforge_mod_latest_files (mod_id, position, record_id) \
         SELECT * FROM UNNEST($1::integer[], $2::integer[], $3::bigint[])",
    )
    .bind(latest.iter().map(|(m, _, _)| *m).collect::<Vec<_>>())
    .bind(latest.iter().map(|(_, p, _)| *p).collect::<Vec<_>>())
    .bind(&record_ids)
    .execute(conn)
    .await?;

    Ok(())
}
