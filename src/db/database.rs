use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::env;

/// 仓库根目录 `migrations/` 下的表结构迁移，编译期嵌入二进制
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub fn get_database_url() -> String {
    env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres@localhost:5432/mcim_backend".to_string())
}

/// 连接 PostgreSQL 并应用未执行的表结构迁移
pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new().connect(&get_database_url()).await?;
    MIGRATOR.run(&pool).await?;

    log::info!("Connected to PostgreSQL successfully!");
    Ok(pool)
}
