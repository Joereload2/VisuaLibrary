use rusqlite::Connection;

use crate::error::InfraError;

/// Embedded, ordered migrations. **Never edit published versions** (D-025).
const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("../../migrations/0001_init.sql")),
    (
        "0002_domain_tables",
        include_str!("../../migrations/0002_domain_tables.sql"),
    ),
    (
        "0003_asset_package_handoff",
        include_str!("../../migrations/0003_asset_package_handoff.sql"),
    ),
];

pub fn migrate(conn: &Connection) -> Result<(), InfraError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    for (version, sql) in MIGRATIONS {
        let already: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [*version],
            |row| row.get(0),
        )?;
        if already {
            continue;
        }

        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        let now = chrono_like_now();
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            [*version, now.as_str()],
        )?;
        // 0001 also creates schema_migrations; insert is idempotent via EXISTS check above.
        tx.commit()?;
    }
    Ok(())
}

fn chrono_like_now() -> String {
    // UTC ISO-8601 without extra deps: use system time via format from Unix if needed.
    // Prefer simple RFC3339 via `time` would add dep; use system clock string.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

pub fn applied_versions(conn: &Connection) -> Result<Vec<String>, InfraError> {
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
