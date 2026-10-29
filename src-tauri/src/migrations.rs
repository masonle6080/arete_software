//! Schema migrations.
//!
//! Every `migrations/NNNN_name.sql` file is one migration. `build.rs` collects
//! them in file name order into [`ALL`]. To change the schema, add one new
//! file with the next number. Never edit a migration that has shipped.
//!
//! The database records each applied migration by name. The schema version is
//! the number of applied migrations, and is also kept in `PRAGMA user_version`.

use rusqlite::Connection;

use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Migration {
    /// File stem, for example `0001_library`. Unique and stable.
    pub name: &'static str,
    pub sql: &'static str,
}

/// All migrations of this app version, in order.
pub static ALL: &[Migration] = include!(concat!(env!("OUT_DIR"), "/migrations.rs"));

/// Brings `conn` up to date with `migrations`. Each migration runs in its own
/// transaction. Refuses a database that has migrations this list lacks.
pub fn apply(conn: &mut Connection, migrations: &[Migration]) -> CoreResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migration (
             name       TEXT PRIMARY KEY NOT NULL,
             applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         );",
    )?;

    let applied = applied_names(conn)?;
    if let Some(unknown) = applied.iter().find(|n| !migrations.iter().any(|m| m.name == *n)) {
        return Err(CoreError::NewerSchema { migration: unknown.clone() });
    }

    for m in migrations.iter().filter(|m| !applied.iter().any(|n| n == m.name)) {
        let tx = conn.transaction()?;
        tx.execute_batch(m.sql)?;
        tx.execute("INSERT INTO schema_migration (name) VALUES (?1)", [m.name])?;
        tx.commit()?;
    }

    let version = schema_version(conn)?;
    conn.pragma_update(None, "user_version", version)?;
    Ok(())
}

/// The number of migrations applied to `conn`.
pub fn schema_version(conn: &Connection) -> CoreResult<u32> {
    Ok(conn.query_row("SELECT COUNT(*) FROM schema_migration", [], |r| r.get(0))?)
}

fn applied_names(conn: &Connection) -> CoreResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM schema_migration")?;
    let names = stmt.query_map([], |r| r.get(0))?.collect::<Result<_, _>>()?;
    Ok(names)
}
