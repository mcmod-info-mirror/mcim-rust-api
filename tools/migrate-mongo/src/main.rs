//! 将 MCIM 的 MongoDB 数据迁移到 PostgreSQL。
//!
//! 数据来源：运行中的 MongoDB（`--mongodb-uri`），或 mongoexport `--jsonArray` 导出的
//! `<集合名>.json` 目录（`--from-json`，如仓库中的 `data/` 测试数据）。
//!
//! 逐个集合读取文档 -> 转成 JSON（日期转 RFC 3339，ObjectId 转十六进制）->
//! 反序列化为 `mcim_rust_api` 的数据库模型 -> 批量写入关系表。
//! 写入以文档为单位整体替换，可重复执行；结束时对比两边文档数，不一致或有文档转换失败则以非零码退出。

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use bson::{Bson, Document, doc};
use chrono::SecondsFormat;
use clap::{Parser, ValueEnum};
use mcim_rust_api::db::database::MIGRATOR;
use mcim_rust_api::db::{curseforge, modrinth, translate};
use mcim_rust_api::models::curseforge::entities as cf;
use mcim_rust_api::models::modrinth::entities as mr;
use mcim_rust_api::models::translate::entities as tr;
use mongodb::Database;
use serde::de::DeserializeOwned;
use serde_json::{Map, Number, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::{AssertSqlSafe, PgConnection, PgPool};

#[derive(Parser)]
#[command(about = "将 MCIM 的 MongoDB 数据迁移到 PostgreSQL")]
struct Args {
    /// MongoDB 连接字符串
    #[arg(long, env = "MONGODB_URI", default_value = "mongodb://localhost:27017")]
    mongodb_uri: String,

    /// MongoDB 数据库名
    #[arg(long, env = "MONGODB_DATABASE", default_value = "mcim_backend")]
    mongodb_database: String,

    /// 改为从该目录下 mongoexport --jsonArray 导出的 <集合名>.json 读取（不连接 MongoDB）；
    /// 缺少文件的集合会被跳过
    #[arg(long, value_name = "DIR")]
    from_json: Option<PathBuf>,

    /// PostgreSQL 连接字符串；表结构迁移会自动应用
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    /// 每批写入的文档数
    #[arg(long, default_value_t = 1000, value_parser = clap::value_parser!(u32).range(1..))]
    batch_size: u32,

    /// 只迁移这些集合（逗号分隔），默认全部
    #[arg(long, value_enum, value_delimiter = ',')]
    collections: Vec<Collection>,

    /// 迁移前清空 PostgreSQL 中的全部数据表
    #[arg(long, conflicts_with = "collections")]
    truncate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Collection {
    #[value(name = "curseforge_categories")]
    CurseforgeCategories,
    #[value(name = "curseforge_mods")]
    CurseforgeMods,
    #[value(name = "curseforge_files")]
    CurseforgeFiles,
    #[value(name = "curseforge_translated")]
    CurseforgeTranslated,
    #[value(name = "modrinth_projects")]
    ModrinthProjects,
    #[value(name = "modrinth_versions")]
    ModrinthVersions,
    #[value(name = "modrinth_files")]
    ModrinthFiles,
    #[value(name = "modrinth_categories")]
    ModrinthCategories,
    #[value(name = "modrinth_loaders")]
    ModrinthLoaders,
    #[value(name = "modrinth_game_versions")]
    ModrinthGameVersions,
    #[value(name = "modrinth_translated")]
    ModrinthTranslated,
}

impl Collection {
    const ALL: [Collection; 11] = [
        Collection::CurseforgeCategories,
        Collection::CurseforgeMods,
        Collection::CurseforgeFiles,
        Collection::CurseforgeTranslated,
        Collection::ModrinthProjects,
        Collection::ModrinthVersions,
        Collection::ModrinthFiles,
        Collection::ModrinthCategories,
        Collection::ModrinthLoaders,
        Collection::ModrinthGameVersions,
        Collection::ModrinthTranslated,
    ];

    /// MongoDB 集合名，同时也是 PostgreSQL 中对应的主表名
    fn name(self) -> &'static str {
        match self {
            Collection::CurseforgeCategories => "curseforge_categories",
            Collection::CurseforgeMods => "curseforge_mods",
            Collection::CurseforgeFiles => "curseforge_files",
            Collection::CurseforgeTranslated => "curseforge_translated",
            Collection::ModrinthProjects => "modrinth_projects",
            Collection::ModrinthVersions => "modrinth_versions",
            Collection::ModrinthFiles => "modrinth_files",
            Collection::ModrinthCategories => "modrinth_categories",
            Collection::ModrinthLoaders => "modrinth_loaders",
            Collection::ModrinthGameVersions => "modrinth_game_versions",
            Collection::ModrinthTranslated => "modrinth_translated",
        }
    }
}

/// PostgreSQL 中全部数据表（不含 sqlx 迁移记录表）
const ALL_TABLES: &str = "curseforge_categories, curseforge_file_records, curseforge_files, \
    curseforge_mods, curseforge_translated, modrinth_projects, modrinth_versions, modrinth_files, \
    modrinth_categories, modrinth_loaders, modrinth_game_versions, modrinth_translated";

/// 一个集合对应的数据库模型及写入方式
trait Target: DeserializeOwned + Sized {
    /// 允许被丢弃的文档字段（如 ObjectId 主键）
    const DROPPED_FIELDS: &'static [&'static str] = &[];
    /// 整个集合一次性整体替换（不分批）
    const FULL_REPLACE: bool = false;

    /// 写入前整理批次（如去重）
    fn prepare(_batch: &mut Vec<Self>) {}

    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error>;
}

impl Target for cf::Category {
    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        curseforge::upsert_categories(conn, batch).await
    }
}

impl Target for cf::Mod {
    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        curseforge::replace_mods(conn, batch).await
    }
}

impl Target for cf::File {
    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        curseforge::replace_files(conn, batch).await
    }
}

impl Target for tr::CurseForgeTranslation {
    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        translate::upsert_curseforge_translations(conn, batch).await
    }
}

impl Target for mr::Project {
    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        modrinth::replace_projects(conn, batch).await
    }
}

impl Target for mr::Version {
    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        modrinth::replace_versions(conn, batch).await
    }
}

impl Target for mr::File {
    /// PostgreSQL 主键为 (sha1, sha512)；MongoDB 中字段顺序不同的 `_id` 会映射到同一行，保留最后一个
    fn prepare(batch: &mut Vec<Self>) {
        let mut seen = std::collections::HashSet::new();
        let mut deduped: Vec<Self> = batch
            .drain(..)
            .rev()
            .filter(|f| seen.insert((f.hashes.sha1.clone(), f.hashes.sha512.clone())))
            .collect();
        deduped.reverse();
        *batch = deduped;
    }

    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        modrinth::upsert_files(conn, batch).await
    }
}

impl Target for mr::Category {
    const DROPPED_FIELDS: &'static [&'static str] = &["_id"];
    const FULL_REPLACE: bool = true;

    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        modrinth::replace_categories(conn, batch).await
    }
}

impl Target for mr::Loader {
    const DROPPED_FIELDS: &'static [&'static str] = &["_id"];
    const FULL_REPLACE: bool = true;

    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        modrinth::replace_loaders(conn, batch).await
    }
}

impl Target for mr::GameVersion {
    const DROPPED_FIELDS: &'static [&'static str] = &["_id"];
    const FULL_REPLACE: bool = true;

    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        modrinth::replace_game_versions(conn, batch).await
    }
}

impl Target for tr::ModrinthTranslation {
    async fn write(conn: &mut PgConnection, batch: &[Self]) -> Result<(), sqlx::Error> {
        translate::upsert_modrinth_translations(conn, batch).await
    }
}

/// 文档来源
enum Source {
    Mongo(Database),
    JsonDir(PathBuf),
}

/// 一个集合的文档流
enum Documents {
    Cursor(Box<mongodb::Cursor<Document>>),
    Loaded(std::vec::IntoIter<Document>),
}

impl Documents {
    async fn next(&mut self) -> Result<Option<Document>> {
        match self {
            Documents::Cursor(cursor) => Ok(if cursor.advance().await? {
                Some(cursor.deserialize_current()?)
            } else {
                None
            }),
            Documents::Loaded(docs) => Ok(docs.next()),
        }
    }
}

impl Source {
    /// 打开集合，返回 (文档数, 文档流)；JSON 目录中没有对应文件时返回 None
    async fn open(&self, name: &str, batch_size: u32) -> Result<Option<(u64, Documents)>> {
        match self {
            Source::Mongo(db) => {
                let collection = db.collection::<Document>(name);
                let count = collection.count_documents(doc! {}).await?;
                // 按自然顺序读取：整体替换的集合（tag 类）在 PostgreSQL 中按写入顺序返回，与原接口一致
                let cursor = collection.find(doc! {}).batch_size(batch_size).await?;
                Ok(Some((count, Documents::Cursor(Box::new(cursor)))))
            }
            Source::JsonDir(dir) => {
                let path = dir.join(format!("{name}.json"));
                if !path.exists() {
                    return Ok(None);
                }
                let text = std::fs::read_to_string(&path)
                    .with_context(|| format!("failed to read {}", path.display()))?;
                let values: Vec<Value> = serde_json::from_str(&text)
                    .with_context(|| format!("{} is not a JSON array", path.display()))?;
                let docs = values
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| match Bson::try_from(value) {
                        Ok(Bson::Document(doc)) => Ok(doc),
                        Ok(other) => bail!("{}[{i}]: not a document: {other:?}", path.display()),
                        Err(e) => Err(e).with_context(|| format!("{}[{i}]", path.display())),
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(Some((
                    docs.len() as u64,
                    Documents::Loaded(docs.into_iter()),
                )))
            }
        }
    }
}

/// 单个集合的迁移结果
struct Report {
    collection: Collection,
    source_count: u64,
    migrated: u64,
    failed: u64,
    target_count: i64,
    /// 未映射到任何列而被丢弃的字段路径 -> 出现次数
    dropped_fields: BTreeMap<String, u64>,
}

impl Report {
    fn ok(&self) -> bool {
        self.failed == 0 && u64::try_from(self.target_count) == Ok(self.source_count)
    }
}

/// BSON -> JSON：日期转 RFC 3339 字符串，ObjectId 转十六进制，整数值的 double 转整数
fn bson_to_json(value: Bson) -> Result<Value> {
    Ok(match value {
        Bson::Null | Bson::Undefined => Value::Null,
        Bson::Boolean(b) => Value::Bool(b),
        Bson::Int32(i) => Value::from(i),
        Bson::Int64(i) => Value::from(i),
        Bson::Double(f) => {
            if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                Value::from(f as i64)
            } else {
                Value::Number(
                    Number::from_f64(f).with_context(|| format!("non-finite double {f}"))?,
                )
            }
        }
        Bson::String(s) => Value::String(s),
        Bson::ObjectId(oid) => Value::String(oid.to_hex()),
        Bson::DateTime(dt) => {
            Value::String(dt.to_chrono().to_rfc3339_opts(SecondsFormat::AutoSi, true))
        }
        Bson::Array(items) => Value::Array(
            items
                .into_iter()
                .map(bson_to_json)
                .collect::<Result<Vec<_>>>()?,
        ),
        Bson::Document(doc) => document_to_json(doc)?,
        other => bail!("unsupported BSON value {other:?}"),
    })
}

fn document_to_json(doc: Document) -> Result<Value> {
    let mut map = Map::with_capacity(doc.len());
    for (key, value) in doc {
        let converted = bson_to_json(value).with_context(|| format!("field `{key}`"))?;
        map.insert(key, converted);
    }
    Ok(Value::Object(map))
}

/// 字段路径，数组下标统一记为 `[]`
fn format_path(path: &serde_ignored::Path) -> String {
    use serde_ignored::Path;
    match path {
        Path::Root => String::new(),
        Path::Seq { parent, .. } => format!("{}[]", format_path(parent)),
        Path::Map { parent, key } => {
            let parent = format_path(parent);
            if parent.is_empty() {
                key.clone()
            } else {
                format!("{parent}.{key}")
            }
        }
        Path::Some { parent }
        | Path::NewtypeStruct { parent }
        | Path::NewtypeVariant { parent } => format_path(parent),
    }
}

/// 转换一个文档；返回 (模型, 被丢弃的字段路径)
fn convert<T: Target>(doc: Document) -> Result<(T, Vec<String>)> {
    let json = document_to_json(doc)?;
    let mut dropped = Vec::new();
    let value = serde_ignored::deserialize(json, |path| {
        let path = format_path(&path);
        if !T::DROPPED_FIELDS.contains(&path.as_str()) {
            dropped.push(path);
        }
    })?;
    Ok((value, dropped))
}

async fn write_batch<T: Target>(pool: &PgPool, batch: &mut Vec<T>) -> Result<()> {
    T::prepare(batch);
    let mut tx = pool.begin().await?;
    T::write(&mut tx, batch).await?;
    tx.commit().await?;
    batch.clear();
    Ok(())
}

async fn migrate<T: Target>(
    source: &Source,
    pool: &PgPool,
    collection: Collection,
    batch_size: usize,
) -> Result<Option<Report>> {
    let name = collection.name();
    let Some((source_count, mut documents)) = source.open(name, batch_size as u32).await? else {
        log::warn!("{name}: no source data, skipped");
        return Ok(None);
    };
    log::info!("{name}: {source_count} documents");

    let mut batch: Vec<T> = Vec::with_capacity(if T::FULL_REPLACE { 0 } else { batch_size });
    let mut migrated = 0u64;
    let mut failed = 0u64;
    let mut dropped_fields: BTreeMap<String, u64> = BTreeMap::new();

    while let Some(doc) = documents.next().await? {
        let id = doc.get("_id").cloned().unwrap_or(Bson::Null);
        match convert::<T>(doc) {
            Ok((value, dropped)) => {
                for path in dropped {
                    *dropped_fields.entry(path).or_default() += 1;
                }
                batch.push(value);
            }
            Err(e) => {
                failed += 1;
                log::error!("{name}: skipping document _id={id}: {e:#}");
                continue;
            }
        }
        if !T::FULL_REPLACE && batch.len() >= batch_size {
            migrated += batch.len() as u64;
            write_batch(pool, &mut batch)
                .await
                .with_context(|| format!("{name}: failed to write batch"))?;
            log::info!("{name}: {migrated}/{source_count}");
        }
    }
    // 整体替换的集合即使为空也要写入，以清空旧数据
    if T::FULL_REPLACE || !batch.is_empty() {
        migrated += batch.len() as u64;
        write_batch(pool, &mut batch)
            .await
            .with_context(|| format!("{name}: failed to write batch"))?;
    }

    let target_count: i64 =
        sqlx::query_scalar(AssertSqlSafe(format!("SELECT count(*) FROM {name}")))
            .fetch_one(pool)
            .await?;
    log::info!("{name}: done, {migrated} migrated, {failed} failed");

    Ok(Some(Report {
        collection,
        source_count,
        migrated,
        failed,
        target_count,
        dropped_fields,
    }))
}

async fn run(args: Args) -> Result<bool> {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&args.database_url)
        .await
        .context("failed to connect to PostgreSQL")?;
    MIGRATOR
        .run(&pool)
        .await
        .context("failed to apply PostgreSQL schema migrations")?;

    let source = match args.from_json {
        Some(dir) => Source::JsonDir(dir),
        None => {
            let client = mongodb::Client::with_uri_str(&args.mongodb_uri)
                .await
                .context("failed to connect to MongoDB")?;
            let mongo = client.database(&args.mongodb_database);
            mongo
                .run_command(doc! { "ping": 1 })
                .await
                .context("failed to ping MongoDB")?;
            Source::Mongo(mongo)
        }
    };

    if args.truncate {
        log::info!("truncating all PostgreSQL data tables");
        sqlx::query(AssertSqlSafe(format!(
            "TRUNCATE {ALL_TABLES} RESTART IDENTITY CASCADE"
        )))
        .execute(&pool)
        .await?;
    }

    let selected: Vec<Collection> = if args.collections.is_empty() {
        Collection::ALL.to_vec()
    } else {
        Collection::ALL
            .into_iter()
            .filter(|c| args.collections.contains(c))
            .collect()
    };
    let batch_size = args.batch_size as usize;

    let mut reports = Vec::with_capacity(selected.len());
    for collection in selected {
        let report = match collection {
            Collection::CurseforgeCategories => {
                migrate::<cf::Category>(&source, &pool, collection, batch_size).await
            }
            Collection::CurseforgeMods => {
                migrate::<cf::Mod>(&source, &pool, collection, batch_size).await
            }
            Collection::CurseforgeFiles => {
                migrate::<cf::File>(&source, &pool, collection, batch_size).await
            }
            Collection::CurseforgeTranslated => {
                migrate::<tr::CurseForgeTranslation>(&source, &pool, collection, batch_size).await
            }
            Collection::ModrinthProjects => {
                migrate::<mr::Project>(&source, &pool, collection, batch_size).await
            }
            Collection::ModrinthVersions => {
                migrate::<mr::Version>(&source, &pool, collection, batch_size).await
            }
            Collection::ModrinthFiles => {
                migrate::<mr::File>(&source, &pool, collection, batch_size).await
            }
            Collection::ModrinthCategories => {
                migrate::<mr::Category>(&source, &pool, collection, batch_size).await
            }
            Collection::ModrinthLoaders => {
                migrate::<mr::Loader>(&source, &pool, collection, batch_size).await
            }
            Collection::ModrinthGameVersions => {
                migrate::<mr::GameVersion>(&source, &pool, collection, batch_size).await
            }
            Collection::ModrinthTranslated => {
                migrate::<tr::ModrinthTranslation>(&source, &pool, collection, batch_size).await
            }
        }?;
        reports.extend(report);
    }

    // 更新统计信息：/statistics 依赖 pg_class.reltuples 估算行数
    sqlx::query("ANALYZE").execute(&pool).await?;

    print!("{}", render_summary(&reports));
    Ok(reports.iter().all(Report::ok))
}

fn render_summary(reports: &[Report]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "\n{:<24} {:>12} {:>12} {:>8} {:>12}  status",
        "collection", "source", "migrated", "failed", "postgresql"
    );
    for r in reports {
        let _ = writeln!(
            out,
            "{:<24} {:>12} {:>12} {:>8} {:>12}  {}",
            r.collection.name(),
            r.source_count,
            r.migrated,
            r.failed,
            r.target_count,
            if r.ok() { "OK" } else { "MISMATCH" }
        );
    }
    for r in reports.iter().filter(|r| !r.dropped_fields.is_empty()) {
        let _ = writeln!(
            out,
            "\n{}: fields not stored in PostgreSQL (path: documents)",
            r.collection.name()
        );
        for (path, count) in &r.dropped_fields {
            let _ = writeln!(out, "  {path}: {count}");
        }
    }
    out
}

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(
        env_logger::Env::new().default_filter_or("info,sqlx::postgres::notice=warn"),
    );

    match run(Args::parse()).await {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            log::error!("migration finished with mismatches, see summary above");
            ExitCode::FAILURE
        }
        Err(e) => {
            log::error!("migration aborted: {e:#}");
            ExitCode::FAILURE
        }
    }
}
