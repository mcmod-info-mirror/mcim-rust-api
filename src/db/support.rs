//! 关系表读写的公共工具。

use std::collections::HashMap;
use std::hash::Hash;

use sqlx::postgres::{PgHasArrayType, PgRow};
use sqlx::{Decode, Encode, FromRow, PgConnection, PgPool, Postgres, Row, Transaction, Type};

/// 只读一致性快照：一份文档分布在多张表里，多次查询必须读到同一时刻的数据
pub async fn read_snapshot(pool: &PgPool) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
    pool.begin_with("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .await
}

/// 子表行：`parent_key` 列为父键，其余列交给 `T`
pub(crate) struct Keyed<K, T> {
    pub key: K,
    pub item: T,
}

impl<'r, K, T> FromRow<'r, PgRow> for Keyed<K, T>
where
    K: Decode<'r, Postgres> + Type<Postgres>,
    T: FromRow<'r, PgRow>,
{
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            key: row.try_get("parent_key")?,
            item: T::from_row(row)?,
        })
    }
}

/// 执行以 `$1 = 父键数组` 为参数、按 (parent_key, position) 排序的子表查询，按父键分组
pub(crate) async fn fetch_grouped<K, T>(
    conn: &mut PgConnection,
    sql: &'static str,
    keys: &[K],
) -> Result<HashMap<K, Vec<T>>, sqlx::Error>
where
    K: for<'r> Decode<'r, Postgres>
        + for<'q> Encode<'q, Postgres>
        + Type<Postgres>
        + PgHasArrayType
        + Eq
        + Hash
        + Send
        + Unpin,
    T: for<'r> FromRow<'r, PgRow> + Send + Unpin,
{
    let mut grouped: HashMap<K, Vec<T>> = HashMap::new();
    if keys.is_empty() {
        return Ok(grouped);
    }
    let rows: Vec<Keyed<K, T>> = sqlx::query_as(sql).bind(keys).fetch_all(conn).await?;
    for row in rows {
        grouped.entry(row.key).or_default().push(row.item);
    }
    Ok(grouped)
}

/// 父键 -> 有序字符串数组（`SELECT parent_key, array_agg(value ORDER BY position) ... GROUP BY parent_key`）
pub(crate) async fn fetch_string_lists<K>(
    conn: &mut PgConnection,
    sql: &'static str,
    keys: &[K],
) -> Result<HashMap<K, Vec<String>>, sqlx::Error>
where
    K: for<'r> Decode<'r, Postgres>
        + for<'q> Encode<'q, Postgres>
        + Type<Postgres>
        + PgHasArrayType
        + Eq
        + Hash
        + Send
        + Unpin,
{
    if keys.is_empty() {
        return Ok(HashMap::new());
    }
    let rows: Vec<(K, Vec<String>)> = sqlx::query_as(sql).bind(keys).fetch_all(conn).await?;
    Ok(rows.into_iter().collect())
}

/// 将 `parent -> [value]` 展开为 UNNEST 所需的 (父键, position, 值) 三列
pub(crate) fn flatten_string_lists<'a, K: Clone + 'a>(
    lists: impl IntoIterator<Item = (K, Option<&'a Vec<String>>)>,
) -> (Vec<K>, Vec<i32>, Vec<&'a str>) {
    let mut parents = Vec::new();
    let mut positions = Vec::new();
    let mut values = Vec::new();
    for (parent, list) in lists {
        for (position, value) in list.into_iter().flatten().enumerate() {
            parents.push(parent.clone());
            positions.push(position as i32);
            values.push(value.as_str());
        }
    }
    (parents, positions, values)
}

/// 为自增主键表预分配 `count` 个 id，便于在一次批量写入中同时写入子表
pub(crate) async fn allocate_ids(
    conn: &mut PgConnection,
    sequence_sql: &'static str,
    count: usize,
) -> Result<Vec<i64>, sqlx::Error> {
    if count == 0 {
        return Ok(Vec::new());
    }
    sqlx::query_scalar(sequence_sql)
        .bind(count as i64)
        .fetch_all(conn)
        .await
}
