use crate::model::lexical_item_detail::Synonyms;
use crate::model::{Sentence, TranslationsSet, WordTranslations};
use axum::http::StatusCode;
use sqlx::SqlitePool;
use tracing::error;

pub async fn get_translations(
    db_pool: &SqlitePool,
    query: &str,
    lang_from_iso3: &str,
    lang_to_iso3: &str,
) -> Result<Option<WordTranslations>, (StatusCode, String)> {
    let uid_from = format!("{lang_from_iso3}-000");
    let uid_to = format!("{lang_to_iso3}-000");
    let query = query.trim();

    let sql = r#"
        WITH src_meanings AS (
          SELECT dnx.mn
          FROM ex
          JOIN lv   ON lv.lv = ex.lv
          JOIN dnx  ON dnx.ex = ex.ex
          WHERE lv.uid = ?1
            AND ex.tt = ?2
        )
        SELECT
          ex_ru.tt               AS txt,
          MAX(COALESCE(d_ru.uq, 0)) AS quality
        FROM src_meanings
        JOIN dnx  AS d_ru  ON d_ru.mn = src_meanings.mn
        JOIN lv   AS lv_ru ON lv_ru.lv = d_ru.lv AND lv_ru.uid = ?3
        JOIN ex   AS ex_ru ON ex_ru.ex = d_ru.ex
        GROUP BY ex_ru.tt
        ORDER BY ex_ru.tt
    "#;

    let rows: Vec<(String, i64)> = sqlx::query_as::<_, (String, i64)>(sql)
        .bind(&uid_from)
        .bind(query)
        .bind(&uid_to)
        .fetch_all(db_pool)
        .await
        .map_err(|e| {
            error!(error = %e, "failed to execute PanLex translation+quality query");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if rows.is_empty() {
        return Ok(None);
    }

    let source = "panlex".to_string();

    let mut translations = Vec::with_capacity(rows.len());
    let mut qualities = Vec::with_capacity(rows.len());
    for (txt, q) in rows {
        translations.push(Sentence::new(txt, lang_to_iso3, &source));
        // PanLex quality is 0–9; clamp to i32 just in case.
        let q = (q as i8).clamp(0, 9);
        qualities.push(q);
    }

    let ts = TranslationsSet {
        original: Sentence::new(query, lang_from_iso3, &source),
        translations,
        translations_qualities: Some(qualities),
    };

    Ok(Some(WordTranslations {
        translations_set: ts,
        source,
    }))
}

pub async fn get_synonyms(
    db_pool: &SqlitePool,
    query: &str,
    lang_from_iso3: &str,
) -> Result<Option<Synonyms>, (StatusCode, String)> {
    let uid_from = format!("{lang_from_iso3}-000");
    let query = query.trim();

    let sql = r#"
        WITH src_expr AS (
          SELECT ex.ex AS src_ex, dnx.mn
          FROM ex
          JOIN lv   ON lv.lv = ex.lv
          JOIN dnx  ON dnx.ex = ex.ex
          WHERE lv.uid = ?1
            AND ex.tt = ?2
        )
        SELECT
          ex_syn.tt                    AS txt,
          MAX(COALESCE(d_syn.uq, 0))   AS quality
        FROM src_expr
        JOIN dnx  AS d_syn  ON d_syn.mn = src_expr.mn
        JOIN lv   AS lv_syn ON lv_syn.lv = d_syn.lv AND lv_syn.uid = ?1
        JOIN ex   AS ex_syn ON ex_syn.ex = d_syn.ex
        WHERE ex_syn.ex NOT IN (SELECT src_ex FROM src_expr)
          AND ex_syn.tt <> ?2
        GROUP BY ex_syn.tt
        ORDER BY ex_syn.tt
    "#;

    let rows: Vec<(String, i64)> = sqlx::query_as::<_, (String, i64)>(sql)
        .bind(&uid_from)
        .bind(query)
        .fetch_all(db_pool)
        .await
        .map_err(|e| {
            error!(error = %e, "failed to execute PanLex synonyms query");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    if rows.is_empty() {
        return Ok(None);
    }

    let source = "panlex".to_string();
    let mut syns = Vec::with_capacity(rows.len());
    let mut quals = Vec::with_capacity(rows.len());
    for (txt, q) in rows {
        syns.push(Sentence::new(txt, lang_from_iso3, &source));
        let q = (q as i8).clamp(0, 9);
        quals.push(q);
    }

    let ts = TranslationsSet {
        original: Sentence::new(query, lang_from_iso3, &source),
        translations: syns,
        translations_qualities: Some(quals),
    };

    Ok(Some(Synonyms {
        translations_set: ts,
        source,
    }))
}

#[cfg(test)]
mod tests {
    use crate::model::lexical_item_detail::Synonyms;
    use crate::model::{Sentence, TranslationsSet, WordTranslations};
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

    async fn new_test_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1) // <- important for :memory:
            .connect("sqlite::memory:")
            .await
            .expect("connect :memory:");
        create_full_schema(&pool).await.expect("schema");
        pool
    }

    async fn create_full_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        const SCHEMA: &str = r#"
CREATE TABLE langvar (
    id integer PRIMARY KEY,
    lang_code text,
    var_code integer,
    uid text,
    meaning integer,
    name_expr integer,
    name_expr_txt text,
    region_expr integer,
    region_expr_txt text,
    script_expr integer,
    script_expr_txt text
);
CREATE TABLE source (
    id integer PRIMARY KEY,
    grp integer,
    label text,
    reg_date text,
    url text,
    isbn text,
    author text,
    title text,
    publisher text,
    year text,
    quality integer,
    note text,
    license text,
    ip_claim text,
    ip_claimant text,
    ip_claimant_email text
);
CREATE TABLE expr (
    id integer PRIMARY KEY,
    langvar integer,
    txt text
);
CREATE TABLE denotationx (
    meaning integer,
    source integer,
    grp integer,
    quality integer,
    expr integer,
    langvar integer
);
CREATE VIEW lv AS SELECT id as lv, lang_code as lc, var_code as vc, uid, meaning as mn, name_expr as ex, name_expr_txt as tt, region_expr as rg, region_expr_txt as rgtt, script_expr as sc, script_expr_txt as sctt FROM langvar
/* lv(lv,lc,vc,uid,mn,ex,tt,rg,rgtt,sc,sctt) */;
CREATE VIEW ex AS SELECT id as ex, langvar as lv, txt as tt FROM expr
/* ex(ex,lv,tt) */;
CREATE VIEW dnx AS SELECT meaning as mn, source as ap, grp as ui, quality as uq, expr as ex, langvar as lv FROM denotationx
/* dnx(mn,ap,ui,uq,ex,lv) */;
CREATE INDEX expr_langvar ON expr (langvar);
CREATE INDEX expr_txt_langvar ON expr (txt, langvar);
CREATE INDEX denotationx_meaning ON denotationx (meaning);
CREATE INDEX denotationx_expr ON denotationx (expr);
CREATE INDEX denotationx_langvar ON denotationx (langvar);
"#;
        for stmt in SCHEMA.split(';') {
            let sql = stmt.trim();
            if !sql.is_empty() {
                sqlx::query(sql).execute(pool).await?;
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn translations_happy_path() {
        let pool = new_test_pool().await;
        // Prepare
        sqlx::query(
            r#"-- noinspection SqlNoDataSourceInspectionForFile
                INSERT INTO langvar(id, lang_code, var_code, uid) VALUES
                  (100,'deu',0,'deu-000'),
                  (300,'eng',0,'eng-000');
                INSERT INTO expr(id, langvar, txt) VALUES
                  (1000,100,'Imker'),
                  (3000,300,'beekeeper'),
                  (3001,300,'apiarist');
                INSERT INTO denotationx(meaning, source, grp, quality, expr, langvar) VALUES
                  (9999, 1, 1, 7, 1000, 100),   -- DE "Imker"
                  (9999, 1, 1, 5, 3000, 300),   -- EN "beekeeper" quality 5
                  (9999, 1, 1, 3, 3000, 300),   -- duplicate lower quality -> MAX keeps 5
                  (9999, 1, 1, 12, 3001, 300);  -- EN "apiarist" -> clamp to 9
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let result = super::get_translations(&pool, " Imker ", "deu", "eng")
            .await
            .expect("ok")
            .expect("some");

        let source = "panlex".to_string();
        let expected = WordTranslations {
            translations_set: TranslationsSet {
                original: Sentence::new("Imker", "deu", &source),
                translations: vec![
                    Sentence::new("apiarist", "eng", &source),
                    Sentence::new("beekeeper", "eng", &source),
                ],
                translations_qualities: Some(vec![9_i8, 5_i8]),
            },
            source: source.clone(),
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn translations_return_none_when_no_match() {
        let pool = new_test_pool().await;
        let out = super::get_translations(&pool, "Nope", "deu", "eng")
            .await
            .expect("ok");
        assert_eq!(out, None);
    }

    #[tokio::test]
    async fn synonyms_happy_path_same_language() {
        let pool = new_test_pool().await;
        sqlx::query(
            r#"-- noinspection SqlNoDataSourceInspectionForFile
            INSERT INTO langvar(id, lang_code, var_code, uid) VALUES
              (100,'deu',0,'deu-000');
            INSERT INTO expr(id, langvar, txt) VALUES
              (1000,100,'Imker'),
              (1001,100,'Bienenhalter'),
              (1002,100,'Bienenzüchter');
            INSERT INTO denotationx(meaning, source, grp, quality, expr, langvar) VALUES
              (9999, 1, 1, 7, 1000, 100),   -- source "Imker"
              (9999, 1, 1, 12, 1001, 100),  -- "Bienenhalter" -> clamp to 9
              (9999, 1, 1, 5,  1002, 100),  -- "Bienenzüchter" quality 5
              (9999, 1, 1, 3,  1002, 100);  -- duplicate lower quality -> MAX keeps 5
        "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let result = super::get_synonyms(&pool, " Imker ", "deu")
            .await
            .expect("ok")
            .expect("some");

        let source = "panlex".to_string();
        let expected = Synonyms {
            translations_set: TranslationsSet {
                original: Sentence::new("Imker", "deu", &source),
                translations: vec![
                    Sentence::new("Bienenhalter", "deu", &source),
                    Sentence::new("Bienenzüchter", "deu", &source),
                ],
                translations_qualities: Some(vec![9_i8, 5_i8]),
            },
            source: source.clone(),
        };

        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn synonyms_none_when_only_source_exists() {
        let pool = new_test_pool().await;
        sqlx::query(
            r#"-- noinspection SqlNoDataSourceInspectionForFile
            INSERT INTO langvar(id, lang_code, var_code, uid) VALUES
              (100,'deu',0,'deu-000');
            INSERT INTO expr(id, langvar, txt) VALUES
              (1000,100,'Imker');
            INSERT INTO denotationx(meaning, source, grp, quality, expr, langvar) VALUES
              (9999, 1, 1, 7, 1000, 100);
        "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let result = super::get_synonyms(&pool, "Imker", "deu")
            .await
            .expect("ok");

        assert_eq!(result, None);
    }
}
