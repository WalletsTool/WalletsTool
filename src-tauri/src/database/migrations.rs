use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{Executor, SqlitePool};

#[derive(Clone, Copy)]
pub struct DbMigration {
    pub version: i64,
    pub name: &'static str,
    pub check_sql: Option<&'static str>,
    pub sql: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct MigrationApplied {
    pub version: i64,
    pub name: String,
    pub checksum: String,
    pub applied: bool,
}

const MIGRATIONS: &[DbMigration] = &[
    DbMigration {
        version: 1,
        name: "chains_add_ecosystem",
        check_sql: Some("SELECT COUNT(*) FROM pragma_table_info('chains') WHERE name = 'ecosystem'"),
        sql: include_str!("../../data/migrations/V0001__chains_add_ecosystem.sql"),
    },
];

pub fn all_migrations() -> &'static [DbMigration] {
    MIGRATIONS
}

pub fn checksum_sql(sql: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sql.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn ensure_migrations_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    pool.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            app_version TEXT NOT NULL,
            applied INTEGER NOT NULL
        )",
    )
    .await?;
    Ok(())
}

pub async fn is_migration_recorded(pool: &SqlitePool, version: i64) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations WHERE version = ?")
        .bind(version)
        .fetch_one(pool)
        .await?;
    Ok(count > 0)
}

#[allow(dead_code)]
pub async fn record_migration(
    pool: &SqlitePool,
    version: i64,
    name: &str,
    checksum: &str,
    app_version: &str,
    applied: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO schema_migrations(version, name, checksum, app_version, applied) VALUES(?, ?, ?, ?, ?)",
    )
    .bind(version)
    .bind(name)
    .bind(checksum)
    .bind(app_version)
    .bind(if applied { 1 } else { 0 })
    .execute(pool)
    .await?;
    Ok(())
}

