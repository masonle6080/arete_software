//! Thesis: a philosophical idea from the founder's life that a Drop expresses.
//!
//! Domain command functions for this area: [`create_thesis`],
//! [`move_thesis`], [`list_theses`], [`get_thesis`]. The lifecycle rules
//! live on [`ThesisStatus`].

mod status;

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::lifecycle;

pub use status::ThesisStatus;

pub type ThesisId = i64;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Thesis {
    pub id: ThesisId,
    pub title: String,
    pub statement: String,
    pub status: ThesisStatus,
    /// The statuses the founder can move this Thesis to by hand now, so the
    /// UI never repeats the lifecycle rules.
    pub manual_moves: Vec<ThesisStatus>,
}

const COLUMNS: &str = "id, title, statement, status";

fn from_row(row: &Row) -> rusqlite::Result<Thesis> {
    let raw: String = row.get(3)?;
    let status = ThesisStatus::from_db(&raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, format!("unknown status {raw}").into())
    })?;
    Ok(Thesis {
        id: row.get(0)?,
        title: row.get(1)?,
        statement: row.get(2)?,
        status,
        manual_moves: status.moves(),
    })
}

/// Creates a Thesis. A new Thesis starts as Seed. The title must not be
/// blank; the statement may be empty.
pub fn create_thesis(lib: &Library, title: &str, statement: &str) -> CoreResult<Thesis> {
    let title = title.trim();
    if title.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Thesis needs a title".into() });
    }
    let db = lib.writer()?;
    db.execute(
        "INSERT INTO thesis (title, statement, status) VALUES (?1, ?2, ?3)",
        params![title, statement.trim(), ThesisStatus::Seed.as_db()],
    )?;
    get_thesis(lib, db.last_insert_rowid())
}

/// Moves a Thesis to `to` by hand. Refuses moves that
/// [`ThesisStatus::move_allowed`] does not allow.
pub fn move_thesis(lib: &Library, id: ThesisId, to: ThesisStatus) -> CoreResult<Thesis> {
    let from = get_thesis(lib, id)?.status;
    lifecycle::move_status(lib, "thesis", id, from, to)?;
    get_thesis(lib, id)
}

/// Sets a Thesis status as a cascade from its Drop (Committed, Released, or
/// back to Developing). Only the Drop area calls this, inside its own
/// transaction; the founder's moves go through [`move_thesis`].
pub(crate) fn set_status_from_drop(conn: &Connection, id: ThesisId, to: ThesisStatus) -> CoreResult<()> {
    conn.execute("UPDATE thesis SET status = ?1 WHERE id = ?2", params![to.as_db(), id])?;
    Ok(())
}

/// One status group of the Thesis list.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThesisGroup {
    pub status: ThesisStatus,
    /// Oldest first.
    pub theses: Vec<Thesis>,
}

/// All Theses, grouped by status. There is one group per status, in
/// lifecycle order, even when a group is empty. Abandoned Theses stay listed.
pub fn list_theses(lib: &Library) -> CoreResult<Vec<ThesisGroup>> {
    let mut stmt = lib.db().prepare(&format!("SELECT {COLUMNS} FROM thesis ORDER BY id"))?;
    let all: Vec<Thesis> = stmt.query_map([], from_row)?.collect::<Result<_, _>>()?;
    Ok(ThesisStatus::ALL
        .into_iter()
        .map(|status| ThesisGroup {
            status,
            theses: all.iter().filter(|t| t.status == status).cloned().collect(),
        })
        .collect())
}

/// Reads one Thesis.
pub fn get_thesis(lib: &Library, id: ThesisId) -> CoreResult<Thesis> {
    lib.db()
        .query_row(&format!("SELECT {COLUMNS} FROM thesis WHERE id = ?1"), [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { record: "Thesis", id })
}
