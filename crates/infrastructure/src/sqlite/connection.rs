use std::path::Path;

use rusqlite::Connection;

use crate::error::InfraError;

/// Open SQLite with Foundation rules: WAL + foreign_keys ON (D-025).
pub fn open_database(path: &Path) -> Result<Connection, InfraError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000i32)?;
    // Verify foreign_keys actually on (some builds ignore silently if mis-set).
    let fk: i32 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
    if fk != 1 {
        return Err(InfraError::Message(
            "PRAGMA foreign_keys no quedó activo".into(),
        ));
    }
    Ok(conn)
}

pub fn pragma_journal_mode(conn: &Connection) -> Result<String, InfraError> {
    let mode: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
    Ok(mode.to_lowercase())
}

pub fn pragma_foreign_keys(conn: &Connection) -> Result<bool, InfraError> {
    let fk: i32 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
    Ok(fk == 1)
}
