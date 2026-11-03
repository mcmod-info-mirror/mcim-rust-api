//! 翻译表读写。

use sqlx::{PgConnection, PgPool};

use crate::models::translate::entities::{CurseForgeTranslation, ModrinthTranslation};

pub async fn get_modrinth_translations(
    pool: &PgPool,
    project_ids: &[String],
) -> Result<Vec<ModrinthTranslation>, sqlx::Error> {
    sqlx::query_as(
        "SELECT project_id, translated, original, need_to_update, translated_at \
         FROM modrinth_translated WHERE project_id = ANY($1) ORDER BY project_id",
    )
    .bind(project_ids)
    .fetch_all(pool)
    .await
}

pub async fn get_curseforge_translations(
    pool: &PgPool,
    mod_ids: &[i32],
) -> Result<Vec<CurseForgeTranslation>, sqlx::Error> {
    sqlx::query_as(
        "SELECT mod_id, translated, original, need_to_update, translated_at \
         FROM curseforge_translated WHERE mod_id = ANY($1) ORDER BY mod_id",
    )
    .bind(mod_ids)
    .fetch_all(pool)
    .await
}

pub async fn upsert_modrinth_translations(
    conn: &mut PgConnection,
    translations: &[ModrinthTranslation],
) -> Result<(), sqlx::Error> {
    if translations.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO modrinth_translated (project_id, translated, original, need_to_update, \
         translated_at) \
         SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[], $4::boolean[], \
             $5::timestamptz[]) \
         ON CONFLICT (project_id) DO UPDATE SET translated = EXCLUDED.translated, \
             original = EXCLUDED.original, need_to_update = EXCLUDED.need_to_update, \
             translated_at = EXCLUDED.translated_at",
    )
    .bind(
        translations
            .iter()
            .map(|t| t.project_id.as_str())
            .collect::<Vec<_>>(),
    )
    .bind(
        translations
            .iter()
            .map(|t| t.translated.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        translations
            .iter()
            .map(|t| t.original.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        translations
            .iter()
            .map(|t| t.need_to_update)
            .collect::<Vec<_>>(),
    )
    .bind(
        translations
            .iter()
            .map(|t| t.translated_at)
            .collect::<Vec<_>>(),
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn upsert_curseforge_translations(
    conn: &mut PgConnection,
    translations: &[CurseForgeTranslation],
) -> Result<(), sqlx::Error> {
    if translations.is_empty() {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO curseforge_translated (mod_id, translated, original, need_to_update, \
         translated_at) \
         SELECT * FROM UNNEST($1::integer[], $2::text[], $3::text[], $4::boolean[], \
             $5::timestamptz[]) \
         ON CONFLICT (mod_id) DO UPDATE SET translated = EXCLUDED.translated, \
             original = EXCLUDED.original, need_to_update = EXCLUDED.need_to_update, \
             translated_at = EXCLUDED.translated_at",
    )
    .bind(translations.iter().map(|t| t.mod_id).collect::<Vec<_>>())
    .bind(
        translations
            .iter()
            .map(|t| t.translated.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        translations
            .iter()
            .map(|t| t.original.as_deref())
            .collect::<Vec<_>>(),
    )
    .bind(
        translations
            .iter()
            .map(|t| t.need_to_update)
            .collect::<Vec<_>>(),
    )
    .bind(
        translations
            .iter()
            .map(|t| t.translated_at)
            .collect::<Vec<_>>(),
    )
    .execute(conn)
    .await?;
    Ok(())
}
