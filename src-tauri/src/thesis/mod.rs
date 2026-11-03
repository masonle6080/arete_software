//! Thesis: a philosophical idea from the founder's life, and the release of
//! clothing (its Drop) that expresses it. One record covers both.
//!
//! Domain command functions for this area: [`create_thesis`],
//! [`move_thesis`], [`list_theses`], [`get_thesis`]. The lifecycle rules
//! live on [`ThesisStatus`]; the guards on a move live in `guard_move`.

mod status;

use rusqlite::{params, OptionalExtension, Row};

use crate::documentary::{self, DocumentaryId, DocumentaryStatus};
use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::lifecycle;
use crate::style;

pub use status::ThesisStatus;

pub type ThesisId = i64;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Thesis {
    pub id: ThesisId,
    pub title: String,
    pub statement: String,
    pub status: ThesisStatus,
    /// The statuses the founder can move this Thesis to now, so the UI never
    /// repeats the lifecycle rules. A move can still be refused by a guard.
    pub manual_moves: Vec<ThesisStatus>,
    /// Whether the founder can add or duplicate Styles on this Thesis now
    /// ([`ThesisStatus::takes_new_styles`]).
    pub can_add_styles: bool,
}

/// Refuses new or duplicated Styles on a Thesis that does not take them.
pub(crate) fn check_takes_new_styles(thesis: &Thesis) -> CoreResult<()> {
    if thesis.can_add_styles {
        return Ok(());
    }
    Err(CoreError::InvalidInput {
        reason: format!(
            "a Thesis in {} takes no new Styles; add Styles only in Developing, Design or Sampling",
            thesis.status.label()
        ),
    })
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
        can_add_styles: status.takes_new_styles(),
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

/// A reason a Thesis move needs the founder's explicit confirmation. Guards
/// that warn but do not block add a variant here.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ThesisWarning {
    /// Releasing the Thesis while its Documentary is not Released. The
    /// Documentary should come out before the Drop.
    #[serde(rename_all = "camelCase")]
    DocumentaryNotReleased { documentary_id: DocumentaryId, title: String, status: DocumentaryStatus },
}

/// The result of [`move_thesis`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "outcome")]
pub enum ThesisMove {
    /// The move went through.
    Moved(Thesis),
    /// Nothing changed. The move goes through only when the founder repeats
    /// it with `confirmed` set.
    #[serde(rename_all = "camelCase")]
    NeedsConfirmation { thesis: Thesis, warnings: Vec<ThesisWarning> },
}

/// Moves a Thesis to `to`. The lifecycle ([`ThesisStatus::move_allowed`])
/// and the hard guards refuse with an error. Warnings stop the move with
/// [`ThesisMove::NeedsConfirmation`] unless `confirmed` is true.
pub fn move_thesis(lib: &Library, id: ThesisId, to: ThesisStatus, confirmed: bool) -> CoreResult<ThesisMove> {
    let thesis = get_thesis(lib, id)?;
    lifecycle::check_move(thesis.status, to)?;
    let warnings = guard_move(lib, &thesis, to)?;
    if !warnings.is_empty() && !confirmed {
        return Ok(ThesisMove::NeedsConfirmation { thesis, warnings });
    }
    lifecycle::move_status(lib, "thesis", id, thesis.status, to)?;
    Ok(ThesisMove::Moved(get_thesis(lib, id)?))
}

/// The checks on a move that the lifecycle allows, beyond the two statuses.
/// A hard rule returns an error (Production needs every Style Approved or
/// Cut). A soft rule returns a warning the founder can confirm past
/// (Released before the Documentary). Add new guards here.
fn guard_move(lib: &Library, thesis: &Thesis, to: ThesisStatus) -> CoreResult<Vec<ThesisWarning>> {
    let mut warnings = Vec::new();
    if to == ThesisStatus::Production {
        let styles: Vec<String> = style::thesis_styles(lib, thesis.id)?
            .into_iter()
            .filter(|s| !s.status.ready_for_production())
            .map(|s| format!("{} ({})", s.name, s.color))
            .collect();
        if !styles.is_empty() {
            return Err(CoreError::StylesNotReady { styles });
        }
    }
    if to == ThesisStatus::Released {
        if let Some(film) = documentary::thesis_documentary(lib, thesis.id)? {
            if film.status != DocumentaryStatus::Released {
                warnings.push(ThesisWarning::DocumentaryNotReleased {
                    documentary_id: film.id,
                    title: film.title,
                    status: film.status,
                });
            }
        }
    }
    Ok(warnings)
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
