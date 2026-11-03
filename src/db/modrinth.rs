//! Modrinth 关系表读写。表结构见 `migrations/`。

use std::collections::HashMap;

use sqlx::{FromRow, PgConnection, PgPool};

use crate::db::support::{
    allocate_ids, fetch_grouped, fetch_string_lists, flatten_string_lists, read_snapshot,
};
use crate::models::modrinth::entities::{
    Category, Dependencies, DonationUrl, File, FileInfo, GalleryItem, GameVersion, License, Loader,
    ModeratorMessage, Project, Version,
};

/// 文件哈希算法；决定按 sha1 还是 sha512 列匹配
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha1,
    Sha512,
}

impl HashAlgorithm {
    /// 只接受 "sha1" / "sha512"，其他算法名在数据库中不可能有匹配
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "sha1" => Some(Self::Sha1),
            "sha512" => Some(Self::Sha512),
            _ => None,
        }
    }
}

macro_rules! project_columns {
    () => {
        "SELECT id, slug, title, description, client_side, server_side, body, status, \
         requested_status, issues_url, source_url, wiki_url, discord_url, project_type, downloads, \
         icon_url, color, thread_id, monetization_status, team, body_url, translated_description, \
         published, updated, approved, queued, followers, sync_at, checked_at \
         FROM modrinth_projects "
    };
}

macro_rules! version_columns {
    () => {
        "SELECT v.id, v.project_id, v.slug, v.name, v.version_number, v.changelog, \
         v.version_type, v.featured, v.status, v.requested_status, v.author_id, v.date_published, \
         v.downloads, v.changelog_url, v.found, v.sync_at FROM modrinth_versions v "
    };
}

macro_rules! file_columns {
    () => {
        "SELECT sha1, sha512, url, filename, \"primary\", size, file_type, version_id, project_id, \
         file_cdn_cached, found, sync_at FROM modrinth_files "
    };
}

/// 每个哈希对应的、同一 Project 下满足 loader / 游戏版本过滤的最新 Version
macro_rules! latest_versions_for_hashes {
    ($hash_column:literal) => {
        concat!(
            "SELECT DISTINCT ON (f.",
            $hash_column,
            ") f.",
            $hash_column,
            ", v.id FROM modrinth_files f \
             JOIN modrinth_versions v ON v.project_id = f.project_id \
             WHERE f.",
            $hash_column,
            " = ANY($1) \
             AND (cardinality($2::text[]) = 0 OR EXISTS ( \
                 SELECT 1 FROM modrinth_version_loaders l \
                 WHERE l.version_id = v.id AND l.loader = ANY($2))) \
             AND (cardinality($3::text[]) = 0 OR EXISTS ( \
                 SELECT 1 FROM modrinth_version_game_versions g \
                 WHERE g.version_id = v.id AND g.game_version = ANY($3))) \
             ORDER BY f.",
            $hash_column,
            ", v.date_published DESC, v.id"
        )
    };
}

const LOADER_SEQUENCE: &str = "SELECT nextval(pg_get_serial_sequence('modrinth_loaders', 'id')) \
                               FROM generate_series(1, $1)";

#[derive(FromRow)]
struct LoaderRow {
    id: i64,
    #[sqlx(flatten)]
    loader: Loader,
}

// ---------------------------------------------------------------------------
// 读取
// ---------------------------------------------------------------------------

async fn fill_projects(
    conn: &mut PgConnection,
    projects: &mut [Project],
) -> Result<(), sqlx::Error> {
    if projects.is_empty() {
        return Ok(());
    }
    let ids: Vec<String> = projects.iter().map(|p| p.id.clone()).collect();

    let mut categories = fetch_string_lists(
        conn,
        "SELECT project_id, array_agg(category ORDER BY position) \
         FROM modrinth_project_categories WHERE project_id = ANY($1) GROUP BY project_id",
        &ids,
    )
    .await?;
    let mut additional_categories = fetch_string_lists(
        conn,
        "SELECT project_id, array_agg(category ORDER BY position) \
         FROM modrinth_project_additional_categories WHERE project_id = ANY($1) GROUP BY project_id",
        &ids,
    )
    .await?;
    let mut versions = fetch_string_lists(
        conn,
        "SELECT project_id, array_agg(version_id ORDER BY position) \
         FROM modrinth_project_versions WHERE project_id = ANY($1) GROUP BY project_id",
        &ids,
    )
    .await?;
    let mut game_versions = fetch_string_lists(
        conn,
        "SELECT project_id, array_agg(game_version ORDER BY position) \
         FROM modrinth_project_game_versions WHERE project_id = ANY($1) GROUP BY project_id",
        &ids,
    )
    .await?;
    let mut loaders = fetch_string_lists(
        conn,
        "SELECT project_id, array_agg(loader ORDER BY position) \
         FROM modrinth_project_loaders WHERE project_id = ANY($1) GROUP BY project_id",
        &ids,
    )
    .await?;
    let mut donation_urls: HashMap<String, Vec<DonationUrl>> = fetch_grouped(
        conn,
        "SELECT project_id AS parent_key, id, platform, url FROM modrinth_project_donation_urls \
         WHERE project_id = ANY($1) ORDER BY project_id, position",
        &ids,
    )
    .await?;
    let mut gallery: HashMap<String, Vec<GalleryItem>> = fetch_grouped(
        conn,
        "SELECT project_id AS parent_key, url, featured, title, description, created, ordering \
         FROM modrinth_project_gallery WHERE project_id = ANY($1) ORDER BY project_id, position",
        &ids,
    )
    .await?;
    let mut licenses: HashMap<String, Vec<License>> = fetch_grouped(
        conn,
        "SELECT project_id AS parent_key, id, name, url FROM modrinth_project_licenses \
         WHERE project_id = ANY($1) ORDER BY project_id",
        &ids,
    )
    .await?;
    let mut moderator_messages: HashMap<String, Vec<ModeratorMessage>> = fetch_grouped(
        conn,
        "SELECT project_id AS parent_key, message, body FROM modrinth_project_moderator_messages \
         WHERE project_id = ANY($1) ORDER BY project_id",
        &ids,
    )
    .await?;

    for p in projects {
        p.categories = Some(categories.remove(&p.id).unwrap_or_default());
        p.additional_categories = Some(additional_categories.remove(&p.id).unwrap_or_default());
        p.versions = Some(versions.remove(&p.id).unwrap_or_default());
        p.game_versions = Some(game_versions.remove(&p.id).unwrap_or_default());
        p.loaders = Some(loaders.remove(&p.id).unwrap_or_default());
        p.donation_urls = Some(donation_urls.remove(&p.id).unwrap_or_default());
        p.gallery = Some(gallery.remove(&p.id).unwrap_or_default());
        p.license = licenses.remove(&p.id).and_then(|v| v.into_iter().next());
        p.moderator_message = moderator_messages
            .remove(&p.id)
            .and_then(|v| v.into_iter().next());
    }
    Ok(())
}

async fn fill_versions(
    conn: &mut PgConnection,
    versions: &mut [Version],
) -> Result<(), sqlx::Error> {
    if versions.is_empty() {
        return Ok(());
    }
    let ids: Vec<String> = versions.iter().map(|v| v.id.clone()).collect();

    let mut dependencies: HashMap<String, Vec<Dependencies>> = fetch_grouped(
        conn,
        "SELECT version_id AS parent_key, dependency_version_id AS version_id, \
         dependency_project_id AS project_id, file_name, dependency_type \
         FROM modrinth_version_dependencies WHERE version_id = ANY($1) \
         ORDER BY version_id, position",
        &ids,
    )
    .await?;
    let mut game_versions = fetch_string_lists(
        conn,
        "SELECT version_id, array_agg(game_version ORDER BY position) \
         FROM modrinth_version_game_versions WHERE version_id = ANY($1) GROUP BY version_id",
        &ids,
    )
    .await?;
    let mut loaders = fetch_string_lists(
        conn,
        "SELECT version_id, array_agg(loader ORDER BY position) \
         FROM modrinth_version_loaders WHERE version_id = ANY($1) GROUP BY version_id",
        &ids,
    )
    .await?;
    let mut files: HashMap<String, Vec<FileInfo>> = fetch_grouped(
        conn,
        "SELECT version_id AS parent_key, sha1, sha512, url, filename, \"primary\", size, file_type \
         FROM modrinth_version_files WHERE version_id = ANY($1) ORDER BY version_id, position",
        &ids,
    )
    .await?;

    for v in versions {
        v.dependencies = Some(dependencies.remove(&v.id).unwrap_or_default());
        v.game_versions = Some(game_versions.remove(&v.id).unwrap_or_default());
        v.loaders = Some(loaders.remove(&v.id).unwrap_or_default());
        v.files = files.remove(&v.id).unwrap_or_default();
    }
    Ok(())
}

/// 已存在的 Project id
pub async fn existing_project_ids(
    pool: &PgPool,
    project_ids: &[String],
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM modrinth_projects WHERE id = ANY($1)")
        .bind(project_ids)
        .fetch_all(pool)
        .await
}

/// 按 id 或 slug 查找单个 Project；id 匹配优先于 slug 匹配
pub async fn get_project_by_id_or_slug(
    pool: &PgPool,
    id_or_slug: &str,
) -> Result<Option<Project>, sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let mut projects: Vec<Project> = sqlx::query_as(concat!(
        project_columns!(),
        "WHERE id = $1 OR slug = $1 ORDER BY (id = $1) DESC, id LIMIT 1"
    ))
    .bind(id_or_slug)
    .fetch_all(&mut *tx)
    .await?;
    fill_projects(&mut tx, &mut projects).await?;
    tx.commit().await?;
    Ok(projects.pop())
}

/// 按 id 或 slug 批量查找 Project，按 id 升序
pub async fn get_projects(
    pool: &PgPool,
    ids_or_slugs: &[String],
) -> Result<Vec<Project>, sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let mut projects: Vec<Project> = sqlx::query_as(concat!(
        project_columns!(),
        "WHERE id = ANY($1) OR slug = ANY($1) ORDER BY id"
    ))
    .bind(ids_or_slugs)
    .fetch_all(&mut *tx)
    .await?;
    fill_projects(&mut tx, &mut projects).await?;
    tx.commit().await?;
    Ok(projects)
}

/// 按 Version id 批量查找，按发布时间降序
pub async fn get_versions(
    pool: &PgPool,
    version_ids: &[String],
) -> Result<Vec<Version>, sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let mut versions: Vec<Version> = sqlx::query_as(concat!(
        version_columns!(),
        "WHERE v.id = ANY($1) ORDER BY v.date_published DESC, v.id"
    ))
    .bind(version_ids)
    .fetch_all(&mut *tx)
    .await?;
    fill_versions(&mut tx, &mut versions).await?;
    tx.commit().await?;
    Ok(versions)
}

/// Project 的全部 Version，按发布时间降序。
/// `game_versions` / `loaders` 为 Some 时要求 Version 至少包含其中一个值。
pub async fn get_project_versions(
    pool: &PgPool,
    project_id: &str,
    featured: Option<bool>,
    game_versions: Option<&[String]>,
    loaders: Option<&[String]>,
) -> Result<Vec<Version>, sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let mut versions: Vec<Version> = sqlx::query_as(concat!(
        version_columns!(),
        "WHERE v.project_id = $1 \
         AND ($2::boolean IS NULL OR v.featured = $2) \
         AND ($3::text[] IS NULL OR EXISTS ( \
             SELECT 1 FROM modrinth_version_game_versions g \
             WHERE g.version_id = v.id AND g.game_version = ANY($3))) \
         AND ($4::text[] IS NULL OR EXISTS ( \
             SELECT 1 FROM modrinth_version_loaders l \
             WHERE l.version_id = v.id AND l.loader = ANY($4))) \
         ORDER BY v.date_published DESC, v.id"
    ))
    .bind(project_id)
    .bind(featured)
    .bind(game_versions)
    .bind(loaders)
    .fetch_all(&mut *tx)
    .await?;
    fill_versions(&mut tx, &mut versions).await?;
    tx.commit().await?;
    Ok(versions)
}

/// 按哈希查找 modrinth_files 文档
pub async fn get_files_by_hashes(
    pool: &PgPool,
    algorithm: HashAlgorithm,
    hashes: &[String],
) -> Result<Vec<File>, sqlx::Error> {
    let sql = match algorithm {
        HashAlgorithm::Sha1 => concat!(file_columns!(), "WHERE sha1 = ANY($1)"),
        HashAlgorithm::Sha512 => concat!(file_columns!(), "WHERE sha512 = ANY($1)"),
    };
    sqlx::query_as(sql).bind(hashes).fetch_all(pool).await
}

/// 哈希 -> 该文件所属 Project 中满足过滤条件的最新 Version id。
/// 过滤条件为空数组时不过滤；非空时要求 Version 至少包含其中一个值。
pub async fn get_latest_version_ids_for_hashes(
    pool: &PgPool,
    algorithm: HashAlgorithm,
    hashes: &[String],
    loaders: &[String],
    game_versions: &[String],
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let sql = match algorithm {
        HashAlgorithm::Sha1 => latest_versions_for_hashes!("sha1"),
        HashAlgorithm::Sha512 => latest_versions_for_hashes!("sha512"),
    };
    sqlx::query_as(sql)
        .bind(hashes)
        .bind(loaders)
        .bind(game_versions)
        .fetch_all(pool)
        .await
}

pub async fn get_categories(pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as(
        "SELECT icon, name, project_type, header, sync_at FROM modrinth_categories ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_loaders(pool: &PgPool) -> Result<Vec<Loader>, sqlx::Error> {
    let mut tx = read_snapshot(pool).await?;
    let rows: Vec<LoaderRow> =
        sqlx::query_as("SELECT id, icon, name, sync_at FROM modrinth_loaders ORDER BY id")
            .fetch_all(&mut *tx)
            .await?;
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let mut project_types = fetch_string_lists(
        &mut tx,
        "SELECT loader_id, array_agg(project_type ORDER BY position) \
         FROM modrinth_loader_supported_project_types WHERE loader_id = ANY($1) GROUP BY loader_id",
        &ids,
    )
    .await?;
    tx.commit().await?;
    Ok(rows
        .into_iter()
        .map(|mut row| {
            row.loader.supported_project_types = project_types.remove(&row.id).unwrap_or_default();
            row.loader
        })
        .collect())
}

pub async fn get_game_versions(pool: &PgPool) -> Result<Vec<GameVersion>, sqlx::Error> {
    sqlx::query_as(
        "SELECT version, version_type, date, major, sync_at FROM modrinth_game_versions ORDER BY id",
    )
    .fetch_all(pool)
    .await
}

/// (version_id, filename) -> size
pub async fn get_version_file_sizes(
    pool: &PgPool,
    version_ids: &[&str],
) -> Result<HashMap<(String, String), i64>, sqlx::Error> {
    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT version_id, filename, size FROM modrinth_version_files WHERE version_id = ANY($1)",
    )
    .bind(version_ids)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(version_id, filename, size)| ((version_id, filename), size))
        .collect())
}

// ---------------------------------------------------------------------------
// 写入：以文档为单位整体替换（先删后插），调用方负责事务；同一批次内主键不可重复
// ---------------------------------------------------------------------------

/// 写入一组 `parent -> [String]` 子表，`sql` 为 (父键 text[], position integer[], 值 text[]) 的 UNNEST 插入
async fn insert_string_lists<'a>(
    conn: &mut PgConnection,
    sql: &'static str,
    lists: impl IntoIterator<Item = (&'a str, Option<&'a Vec<String>>)>,
) -> Result<(), sqlx::Error> {
    let (parents, positions, values) = flatten_string_lists(lists);
    sqlx::query(sql)
        .bind(parents)
        .bind(positions)
        .bind(values)
        .execute(conn)
        .await?;
    Ok(())
}

/// 整体替换 modrinth_projects 文档
pub async fn replace_projects(
    conn: &mut PgConnection,
    projects: &[Project],
) -> Result<(), sqlx::Error> {
    if projects.is_empty() {
        return Ok(());
    }
    let ids: Vec<&str> = projects.iter().map(|p| p.id.as_str()).collect();
    sqlx::query("DELETE FROM modrinth_projects WHERE id = ANY($1)")
        .bind(&ids)
        .execute(&mut *conn)
        .await?;

    sqlx::query(
        "INSERT INTO modrinth_projects (id, slug, title, description, client_side, server_side, \
         body, status, requested_status, issues_url, source_url, wiki_url, discord_url, \
         project_type, downloads, icon_url, color, thread_id, monetization_status, team, \
         body_url, translated_description, published, updated, approved, queued, followers, \
         sync_at, checked_at) \
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], \
             $6::text[], $7::text[], $8::text[], $9::text[], $10::text[], $11::text[], \
             $12::text[], $13::text[], $14::text[], $15::bigint[], $16::text[], $17::bigint[], \
             $18::text[], $19::text[], $20::text[], $21::text[], $22::text[], \
             $23::timestamptz[], $24::timestamptz[], $25::timestamptz[], $26::timestamptz[], \
             $27::bigint[], $28::timestamptz[], $29::timestamptz[])",
    )
    .bind(&ids)
    .bind(projects.iter().map(|p| p.slug.as_str()).collect::<Vec<_>>())
    .bind(
        projects
            .iter()
            .map(|p| p.title.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.description.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.client_side.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.server_side.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.body.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.status.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.requested_status.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.issues_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.source_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.wiki_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.discord_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.project_type.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(projects.iter().map(|p| p.downloads).collect::<Vec<_>>())
    .bind(
        projects
            .iter()
            .map(|p| p.icon_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(projects.iter().map(|p| p.color).collect::<Vec<_>>())
    .bind(
        projects
            .iter()
            .map(|p| p.thread_id.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.monetization_status.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(projects.iter().map(|p| p.team.as_str()).collect::<Vec<_>>())
    .bind(
        projects
            .iter()
            .map(|p| p.body_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        projects
            .iter()
            .map(|p| p.translated_description.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(projects.iter().map(|p| p.published).collect::<Vec<_>>())
    .bind(projects.iter().map(|p| p.updated).collect::<Vec<_>>())
    .bind(projects.iter().map(|p| p.approved).collect::<Vec<_>>())
    .bind(projects.iter().map(|p| p.queued).collect::<Vec<_>>())
    .bind(
        projects
            .iter()
            .map(|p| i64::from(p.followers))
            .collect::<Vec<_>>(),
    )
    .bind(projects.iter().map(|p| p.sync_at).collect::<Vec<_>>())
    .bind(projects.iter().map(|p| p.checked_at).collect::<Vec<_>>())
    .execute(&mut *conn)
    .await?;

    insert_string_lists(
        conn,
        "INSERT INTO modrinth_project_categories (project_id, position, category) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[])",
        projects
            .iter()
            .map(|p| (p.id.as_str(), p.categories.as_ref())),
    )
    .await?;
    insert_string_lists(
        conn,
        "INSERT INTO modrinth_project_additional_categories (project_id, position, category) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[])",
        projects
            .iter()
            .map(|p| (p.id.as_str(), p.additional_categories.as_ref())),
    )
    .await?;
    insert_string_lists(
        conn,
        "INSERT INTO modrinth_project_versions (project_id, position, version_id) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[])",
        projects
            .iter()
            .map(|p| (p.id.as_str(), p.versions.as_ref())),
    )
    .await?;
    insert_string_lists(
        conn,
        "INSERT INTO modrinth_project_game_versions (project_id, position, game_version) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[])",
        projects
            .iter()
            .map(|p| (p.id.as_str(), p.game_versions.as_ref())),
    )
    .await?;
    insert_string_lists(
        conn,
        "INSERT INTO modrinth_project_loaders (project_id, position, loader) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[])",
        projects.iter().map(|p| (p.id.as_str(), p.loaders.as_ref())),
    )
    .await?;

    // donation_urls
    let donations: Vec<(&str, i32, &DonationUrl)> = projects
        .iter()
        .flat_map(|p| {
            p.donation_urls
                .iter()
                .flatten()
                .enumerate()
                .map(move |(i, d)| (p.id.as_str(), i as i32, d))
        })
        .collect();
    sqlx::query(
        "INSERT INTO modrinth_project_donation_urls (project_id, position, id, platform, url) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[], $4::text[], $5::text[])",
    )
    .bind(donations.iter().map(|(p, _, _)| *p).collect::<Vec<_>>())
    .bind(donations.iter().map(|(_, i, _)| *i).collect::<Vec<_>>())
    .bind(
        donations
            .iter()
            .map(|(_, _, d)| d.id.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        donations
            .iter()
            .map(|(_, _, d)| d.platform.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        donations
            .iter()
            .map(|(_, _, d)| d.url.as_deref())
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // gallery
    let gallery: Vec<(&str, i32, &GalleryItem)> = projects
        .iter()
        .flat_map(|p| {
            p.gallery
                .iter()
                .flatten()
                .enumerate()
                .map(move |(i, g)| (p.id.as_str(), i as i32, g))
        })
        .collect();
    sqlx::query(
        "INSERT INTO modrinth_project_gallery (project_id, position, url, featured, title, \
         description, created, ordering) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[], $4::boolean[], $5::text[], \
             $6::text[], $7::timestamptz[], $8::bigint[])",
    )
    .bind(gallery.iter().map(|(p, _, _)| *p).collect::<Vec<_>>())
    .bind(gallery.iter().map(|(_, i, _)| *i).collect::<Vec<_>>())
    .bind(
        gallery
            .iter()
            .map(|(_, _, g)| g.url.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        gallery
            .iter()
            .map(|(_, _, g)| g.featured)
            .collect::<Vec<_>>(),
    )
    .bind(
        gallery
            .iter()
            .map(|(_, _, g)| g.title.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        gallery
            .iter()
            .map(|(_, _, g)| g.description.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        gallery
            .iter()
            .map(|(_, _, g)| g.created)
            .collect::<Vec<_>>(),
    )
    .bind(
        gallery
            .iter()
            .map(|(_, _, g)| g.ordering)
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // license
    let licenses: Vec<(&str, &License)> = projects
        .iter()
        .filter_map(|p| Some((p.id.as_str(), p.license.as_ref()?)))
        .collect();
    sqlx::query(
        "INSERT INTO modrinth_project_licenses (project_id, id, name, url) \
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[])",
    )
    .bind(licenses.iter().map(|(p, _)| *p).collect::<Vec<_>>())
    .bind(
        licenses
            .iter()
            .map(|(_, l)| l.id.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        licenses
            .iter()
            .map(|(_, l)| l.name.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        licenses
            .iter()
            .map(|(_, l)| l.url.as_deref())
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    // moderator_message
    let messages: Vec<(&str, &ModeratorMessage)> = projects
        .iter()
        .filter_map(|p| Some((p.id.as_str(), p.moderator_message.as_ref()?)))
        .collect();
    sqlx::query(
        "INSERT INTO modrinth_project_moderator_messages (project_id, message, body) \
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])",
    )
    .bind(messages.iter().map(|(p, _)| *p).collect::<Vec<_>>())
    .bind(
        messages
            .iter()
            .map(|(_, m)| m.message.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        messages
            .iter()
            .map(|(_, m)| m.body.as_deref())
            .collect::<Vec<_>>(),
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// 整体替换 modrinth_versions 文档
pub async fn replace_versions(
    conn: &mut PgConnection,
    versions: &[Version],
) -> Result<(), sqlx::Error> {
    if versions.is_empty() {
        return Ok(());
    }
    let ids: Vec<&str> = versions.iter().map(|v| v.id.as_str()).collect();
    sqlx::query("DELETE FROM modrinth_versions WHERE id = ANY($1)")
        .bind(&ids)
        .execute(&mut *conn)
        .await?;

    sqlx::query(
        "INSERT INTO modrinth_versions (id, project_id, slug, name, version_number, changelog, \
         version_type, featured, status, requested_status, author_id, date_published, downloads, \
         changelog_url, found, sync_at) \
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], \
             $6::text[], $7::text[], $8::boolean[], $9::text[], $10::text[], $11::text[], \
             $12::timestamptz[], $13::bigint[], $14::text[], $15::boolean[], $16::timestamptz[])",
    )
    .bind(&ids)
    .bind(
        versions
            .iter()
            .map(|v| v.project_id.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        versions
            .iter()
            .map(|v| v.slug.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        versions
            .iter()
            .map(|v| v.name.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        versions
            .iter()
            .map(|v| v.version_number.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        versions
            .iter()
            .map(|v| v.changelog.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        versions
            .iter()
            .map(|v| v.version_type.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(versions.iter().map(|v| v.featured).collect::<Vec<_>>())
    .bind(
        versions
            .iter()
            .map(|v| v.status.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        versions
            .iter()
            .map(|v| v.requested_status.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        versions
            .iter()
            .map(|v| v.author_id.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        versions
            .iter()
            .map(|v| v.date_published)
            .collect::<Vec<_>>(),
    )
    .bind(versions.iter().map(|v| v.downloads).collect::<Vec<_>>())
    .bind(
        versions
            .iter()
            .map(|v| v.changelog_url.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(versions.iter().map(|v| v.found).collect::<Vec<_>>())
    .bind(versions.iter().map(|v| v.sync_at).collect::<Vec<_>>())
    .execute(&mut *conn)
    .await?;

    // dependencies
    let deps: Vec<(&str, i32, &Dependencies)> = versions
        .iter()
        .flat_map(|v| {
            v.dependencies
                .iter()
                .flatten()
                .enumerate()
                .map(move |(i, d)| (v.id.as_str(), i as i32, d))
        })
        .collect();
    sqlx::query(
        "INSERT INTO modrinth_version_dependencies (version_id, position, dependency_version_id, \
         dependency_project_id, file_name, dependency_type) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[], $4::text[], $5::text[], \
             $6::text[])",
    )
    .bind(deps.iter().map(|(v, _, _)| *v).collect::<Vec<_>>())
    .bind(deps.iter().map(|(_, i, _)| *i).collect::<Vec<_>>())
    .bind(
        deps.iter()
            .map(|(_, _, d)| d.version_id.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        deps.iter()
            .map(|(_, _, d)| d.project_id.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        deps.iter()
            .map(|(_, _, d)| d.file_name.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        deps.iter()
            .map(|(_, _, d)| d.dependency_type.as_str())
            .collect::<Vec<_>>(),
    )
    .execute(&mut *conn)
    .await?;

    insert_string_lists(
        conn,
        "INSERT INTO modrinth_version_game_versions (version_id, position, game_version) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[])",
        versions
            .iter()
            .map(|v| (v.id.as_str(), v.game_versions.as_ref())),
    )
    .await?;
    insert_string_lists(
        conn,
        "INSERT INTO modrinth_version_loaders (version_id, position, loader) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[])",
        versions.iter().map(|v| (v.id.as_str(), v.loaders.as_ref())),
    )
    .await?;

    // files
    let files: Vec<(&str, i32, &FileInfo)> = versions
        .iter()
        .flat_map(|v| {
            v.files
                .iter()
                .enumerate()
                .map(move |(i, f)| (v.id.as_str(), i as i32, f))
        })
        .collect();
    sqlx::query(
        "INSERT INTO modrinth_version_files (version_id, position, sha1, sha512, url, filename, \
         \"primary\", size, file_type) \
         SELECT * FROM UNNEST($1::text[], $2::integer[], $3::text[], $4::text[], $5::text[], \
             $6::text[], $7::boolean[], $8::bigint[], $9::text[])",
    )
    .bind(files.iter().map(|(v, _, _)| *v).collect::<Vec<_>>())
    .bind(files.iter().map(|(_, i, _)| *i).collect::<Vec<_>>())
    .bind(
        files
            .iter()
            .map(|(_, _, f)| f.hashes.sha1.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        files
            .iter()
            .map(|(_, _, f)| f.hashes.sha512.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        files
            .iter()
            .map(|(_, _, f)| f.url.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        files
            .iter()
            .map(|(_, _, f)| f.filename.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(files.iter().map(|(_, _, f)| f.primary).collect::<Vec<_>>())
    .bind(files.iter().map(|(_, _, f)| f.size).collect::<Vec<_>>())
    .bind(
        files
            .iter()
            .map(|(_, _, f)| f.file_type.as_deref())
            .collect::<Vec<_>>(),
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// 插入或更新 modrinth_files 文档（主键为 sha1 + sha512）
pub async fn upsert_files(conn: &mut PgConnection, files: &[File]) -> Result<(), sqlx::Error> {
    if files.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO modrinth_files (sha1, sha512, url, filename, \"primary\", size, file_type, \
         version_id, project_id, file_cdn_cached, found, sync_at) \
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::boolean[], \
             $6::bigint[], $7::text[], $8::text[], $9::text[], $10::boolean[], $11::boolean[], \
             $12::timestamptz[]) \
         ON CONFLICT (sha1, sha512) DO UPDATE SET url = EXCLUDED.url, \
             filename = EXCLUDED.filename, \"primary\" = EXCLUDED.\"primary\", \
             size = EXCLUDED.size, file_type = EXCLUDED.file_type, \
             version_id = EXCLUDED.version_id, project_id = EXCLUDED.project_id, \
             file_cdn_cached = EXCLUDED.file_cdn_cached, found = EXCLUDED.found, \
             sync_at = EXCLUDED.sync_at",
    )
    .bind(
        files
            .iter()
            .map(|f| f.hashes.sha1.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        files
            .iter()
            .map(|f| f.hashes.sha512.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(files.iter().map(|f| f.url.as_str()).collect::<Vec<_>>())
    .bind(
        files
            .iter()
            .map(|f| f.filename.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(files.iter().map(|f| f.primary).collect::<Vec<_>>())
    .bind(files.iter().map(|f| f.size).collect::<Vec<_>>())
    .bind(
        files
            .iter()
            .map(|f| f.file_type.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        files
            .iter()
            .map(|f| f.version_id.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        files
            .iter()
            .map(|f| f.project_id.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(files.iter().map(|f| f.file_cdn_cached).collect::<Vec<_>>())
    .bind(files.iter().map(|f| f.found).collect::<Vec<_>>())
    .bind(files.iter().map(|f| f.sync_at).collect::<Vec<_>>())
    .execute(conn)
    .await?;
    Ok(())
}

/// 以 `categories` 整体替换 modrinth_categories（保持传入顺序）
pub async fn replace_categories(
    conn: &mut PgConnection,
    categories: &[Category],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM modrinth_categories")
        .execute(&mut *conn)
        .await?;
    sqlx::query(
        "INSERT INTO modrinth_categories (icon, name, project_type, header, sync_at) \
         SELECT icon, name, project_type, header, sync_at FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::timestamptz[]) \
         WITH ORDINALITY AS c(icon, name, project_type, header, sync_at, ord) ORDER BY ord",
    )
    .bind(categories.iter().map(|c| c.icon.as_str()).collect::<Vec<_>>())
    .bind(categories.iter().map(|c| c.name.as_str()).collect::<Vec<_>>())
    .bind(categories.iter().map(|c| c.project_type.as_deref()).collect::<Vec<_>>())
    .bind(categories.iter().map(|c| c.header.as_str()).collect::<Vec<_>>())
    .bind(categories.iter().map(|c| c.sync_at).collect::<Vec<_>>())
    .execute(conn)
    .await?;
    Ok(())
}

/// 以 `loaders` 整体替换 modrinth_loaders（保持传入顺序）
pub async fn replace_loaders(
    conn: &mut PgConnection,
    loaders: &[Loader],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM modrinth_loaders")
        .execute(&mut *conn)
        .await?;
    let mut ids = allocate_ids(conn, LOADER_SEQUENCE, loaders.len()).await?;
    // 序列值递增，排序后与传入顺序一一对应，接口按 id 排序即可还原顺序
    ids.sort_unstable();
    sqlx::query(
        "INSERT INTO modrinth_loaders (id, icon, name, sync_at) \
         SELECT * FROM UNNEST($1::bigint[], $2::text[], $3::text[], $4::timestamptz[])",
    )
    .bind(&ids)
    .bind(loaders.iter().map(|l| l.icon.as_str()).collect::<Vec<_>>())
    .bind(loaders.iter().map(|l| l.name.as_str()).collect::<Vec<_>>())
    .bind(loaders.iter().map(|l| l.sync_at).collect::<Vec<_>>())
    .execute(&mut *conn)
    .await?;

    let (parents, positions, values) = flatten_string_lists(
        ids.iter()
            .copied()
            .zip(loaders.iter().map(|l| Some(&l.supported_project_types))),
    );
    sqlx::query(
        "INSERT INTO modrinth_loader_supported_project_types (loader_id, position, project_type) \
         SELECT * FROM UNNEST($1::bigint[], $2::integer[], $3::text[])",
    )
    .bind(parents)
    .bind(positions)
    .bind(values)
    .execute(conn)
    .await?;
    Ok(())
}

/// 以 `game_versions` 整体替换 modrinth_game_versions（保持传入顺序）
pub async fn replace_game_versions(
    conn: &mut PgConnection,
    game_versions: &[GameVersion],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM modrinth_game_versions")
        .execute(&mut *conn)
        .await?;
    sqlx::query(
        "INSERT INTO modrinth_game_versions (version, version_type, date, major, sync_at) \
         SELECT version, version_type, date, major, sync_at FROM UNNEST($1::text[], $2::text[], $3::timestamptz[], $4::boolean[], \
             $5::timestamptz[]) \
         WITH ORDINALITY AS g(version, version_type, date, major, sync_at, ord) ORDER BY ord",
    )
    .bind(game_versions.iter().map(|g| g.version.as_str()).collect::<Vec<_>>())
    .bind(game_versions.iter().map(|g| g.version_type.as_str()).collect::<Vec<_>>())
    .bind(game_versions.iter().map(|g| g.date).collect::<Vec<_>>())
    .bind(game_versions.iter().map(|g| g.major).collect::<Vec<_>>())
    .bind(game_versions.iter().map(|g| g.sync_at).collect::<Vec<_>>())
    .execute(conn)
    .await?;
    Ok(())
}
