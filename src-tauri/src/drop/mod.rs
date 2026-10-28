//! Drop: a release of clothing made from exactly one Thesis.
//!
//! Domain command functions for this area: [`create_drop`], [`get_drop`].
//! The stage rules live on [`DropStage`].

mod stage;

use rusqlite::{params, OptionalExtension, Row};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::style;
use crate::thesis::{self, ThesisId, ThesisStatus};

pub use stage::DropStage;

pub type DropId = i64;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Drop {
    pub id: DropId,
    pub thesis_id: ThesisId,
    pub name: String,
    pub stage: DropStage,
    /// The stages the lifecycle lets this Drop move to now, so the UI never
    /// repeats the rules. A move can still be refused by a guard.
    pub moves: Vec<DropStage>,
}

const COLUMNS: &str = "id, thesis_id, name, stage";

fn from_row(row: &Row) -> rusqlite::Result<Drop> {
    let raw: String = row.get(3)?;
    let stage = DropStage::from_db(&raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, format!("unknown stage {raw}").into())
    })?;
    Ok(Drop { id: row.get(0)?, thesis_id: row.get(1)?, name: row.get(2)?, stage, moves: stage.moves() })
}

/// A reason a Drop move needs the founder's explicit confirmation. Guards
/// that warn but do not block add a variant here (for example, releasing
/// before the Thesis's Documentary is Released).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum DropWarning {}

/// The result of [`move_drop`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "outcome")]
pub enum DropMove {
    /// The move went through, with its Thesis cascade.
    Moved(Drop),
    /// Nothing changed. The move goes through only when the founder repeats
    /// it with `confirmed` set.
    #[serde(rename_all = "camelCase")]
    NeedsConfirmation { drop: Drop, warnings: Vec<DropWarning> },
}

/// Moves a Drop to `to`. The lifecycle ([`DropStage::move_allowed`]) and the
/// hard guards refuse with an error. Warnings stop the move with
/// [`DropMove::NeedsConfirmation`] unless `confirmed` is true.
pub fn move_drop(lib: &Library, id: DropId, to: DropStage, confirmed: bool) -> CoreResult<DropMove> {
    let drop = get_drop(lib, id)?;
    if !drop.stage.move_allowed(to) {
        return Err(CoreError::StatusMoveRefused { what: "Drop", from: drop.stage.label(), to: to.label() });
    }
    let warnings = guard_move(lib, &drop, to)?;
    if !warnings.is_empty() && !confirmed {
        return Ok(DropMove::NeedsConfirmation { drop, warnings });
    }
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE drop_ SET stage = ?1 WHERE id = ?2", params![to.as_db(), id])?;
    if let Some(status) = to.thesis_cascade() {
        thesis::set_status_from_drop(&tx, drop.thesis_id, status)?;
    }
    tx.commit()?;
    Ok(DropMove::Moved(get_drop(lib, id)?))
}

/// The checks on a move that the lifecycle allows, beyond the two stages.
/// A hard rule returns an error (for example, Production needs every Style
/// Approved or Cut). A soft rule returns a warning the founder can confirm
/// past. Add new guards here.
fn guard_move(lib: &Library, drop: &Drop, to: DropStage) -> CoreResult<Vec<DropWarning>> {
    if to == DropStage::Production {
        let styles: Vec<String> = style::drop_styles(lib, drop.id)?
            .into_iter()
            .filter(|s| !s.status.ready_for_production())
            .map(|s| format!("{} ({})", s.name, s.color))
            .collect();
        if !styles.is_empty() {
            return Err(CoreError::StylesNotReady { styles });
        }
    }
    Ok(Vec::new())
}

/// Creates a Drop from a Thesis and sets the Thesis to Committed. A new Drop
/// starts in Design. The name must not be blank.
pub fn create_drop(lib: &Library, thesis_id: ThesisId, name: &str) -> CoreResult<Drop> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::InvalidInput { reason: "a Drop needs a name".into() });
    }
    let thesis = thesis::get_thesis(lib, thesis_id)?;
    if active_drop(lib, thesis_id)?.is_some() {
        return Err(CoreError::ActiveDropExists { thesis_id });
    }
    if !thesis.status.can_take_drop() {
        return Err(CoreError::StatusMoveRefused {
            what: "Thesis",
            from: thesis.status.label(),
            to: ThesisStatus::Committed.label(),
        });
    }
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO drop_ (thesis_id, name, stage) VALUES (?1, ?2, ?3)",
        params![thesis_id, name, DropStage::Design.as_db()],
    )?;
    let id = tx.last_insert_rowid();
    thesis::set_status_from_drop(&tx, thesis_id, ThesisStatus::Committed)?;
    tx.commit()?;
    get_drop(lib, id)
}

/// The Drops of one Thesis, as the Thesis view shows them.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThesisDrops {
    /// Every Drop of the Thesis, Cancelled ones included, oldest first. At
    /// most one is not Cancelled.
    pub drops: Vec<Drop>,
    /// Whether the founder can make a new Drop from this Thesis now, so the
    /// UI never repeats the rule.
    pub can_create_drop: bool,
}

/// The Drops of one Thesis, and whether a new one can be made.
pub fn thesis_drops(lib: &Library, thesis_id: ThesisId) -> CoreResult<ThesisDrops> {
    let thesis = thesis::get_thesis(lib, thesis_id)?;
    let mut stmt = lib.db().prepare(&format!("SELECT {COLUMNS} FROM drop_ WHERE thesis_id = ?1 ORDER BY id"))?;
    let drops: Vec<Drop> = stmt.query_map([thesis_id], from_row)?.collect::<Result<_, _>>()?;
    let has_active = drops.iter().any(|d| d.stage != DropStage::Cancelled);
    Ok(ThesisDrops { can_create_drop: !has_active && thesis.status.can_take_drop(), drops })
}

/// The one Drop of a Thesis that is not Cancelled, if any.
fn active_drop(lib: &Library, thesis_id: ThesisId) -> CoreResult<Option<Drop>> {
    Ok(lib
        .db()
        .query_row(
            &format!("SELECT {COLUMNS} FROM drop_ WHERE thesis_id = ?1 AND stage <> ?2"),
            params![thesis_id, DropStage::Cancelled.as_db()],
            from_row,
        )
        .optional()?)
}

/// Reads one Drop.
pub fn get_drop(lib: &Library, id: DropId) -> CoreResult<Drop> {
    lib.db()
        .query_row(&format!("SELECT {COLUMNS} FROM drop_ WHERE id = ?1"), [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { record: "Drop", id })
}
