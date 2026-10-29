//! Sample: a physical piece that a Vendor makes for a Style before
//! production (GLOSSARY: Sample). Each Sample is one numbered round for its
//! Style and Vendor. It holds photos and feedback, and ends approved,
//! revised, or rejected. Marking a Sample revised creates the next round for
//! the same Style and Vendor, with the feedback copied forward. Dated notes
//! live in the shared Notes record ([`crate::note`], owner
//! `NoteOwner::Sample`).
//!
//! Domain command functions for this area: [`request_sample`],
//! [`get_sample`], [`style_samples`], [`set_sample_feedback`],
//! [`attach_sample_photos`], [`decide_sample`]. Fit notes on Samples of
//! Tech Pack Styles are in [`fit`]: [`sample_fit`], [`set_fit_note`].

mod fit;

pub use fit::{sample_fit, set_fit_note, FitCell, FitRow, SampleFit};
pub(crate) use fit::{rounds_measuring_point_of_measure, rounds_measuring_size};

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::media::{self, MediaRef};
use crate::style::{self, StyleId};
use crate::vendor::{self, VendorId};

pub type SampleId = i64;

/// Where a Sample stands. A Sample starts Pending and is decided once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SampleOutcome {
    Pending,
    Approved,
    Revised,
    Rejected,
}

impl SampleOutcome {
    pub const ALL: [SampleOutcome; 4] =
        [SampleOutcome::Pending, SampleOutcome::Approved, SampleOutcome::Revised, SampleOutcome::Rejected];

    /// The founder-facing name, for example "Approved".
    pub fn label(self) -> &'static str {
        match self {
            SampleOutcome::Pending => "Pending",
            SampleOutcome::Approved => "Approved",
            SampleOutcome::Revised => "Revised",
            SampleOutcome::Rejected => "Rejected",
        }
    }

    fn as_db(self) -> &'static str {
        match self {
            SampleOutcome::Pending => "pending",
            SampleOutcome::Approved => "approved",
            SampleOutcome::Revised => "revised",
            SampleOutcome::Rejected => "rejected",
        }
    }

    fn from_db(s: &str) -> Option<SampleOutcome> {
        SampleOutcome::ALL.into_iter().find(|o| o.as_db() == s)
    }
}

/// One photo of a Sample.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplePhoto {
    pub id: i64,
    pub media: MediaRef,
    /// The name the file had when the founder attached it.
    pub file_name: String,
    /// Where the stored copy is now, inside the library's media folder.
    pub path: PathBuf,
}

/// A Sample as the founder sees it. Photos are in the order added.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub id: SampleId,
    pub style_id: StyleId,
    pub vendor_id: VendorId,
    /// The round number for this Style and Vendor, from 1.
    pub round: u32,
    pub outcome: SampleOutcome,
    pub feedback: String,
    pub photos: Vec<SamplePhoto>,
}

const COLUMNS: &str = "id, style_id, vendor_id, round, outcome, feedback";

fn from_row(row: &Row) -> rusqlite::Result<Sample> {
    let raw: String = row.get(4)?;
    let outcome = SampleOutcome::from_db(&raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, format!("unknown outcome {raw}").into())
    })?;
    Ok(Sample {
        id: row.get(0)?,
        style_id: row.get(1)?,
        vendor_id: row.get(2)?,
        round: row.get(3)?,
        outcome,
        feedback: row.get(5)?,
        photos: Vec::new(),
    })
}

/// Requests a Sample of a Style from a Vendor. The first Sample for a Style
/// and Vendor is round 1; it starts Pending with no feedback.
pub fn request_sample(lib: &Library, style_id: StyleId, vendor_id: VendorId) -> CoreResult<Sample> {
    style::get_style(lib, style_id)?;
    vendor::get_vendor(lib, vendor_id)?;
    let conn = lib.writer()?;
    let round = next_round(conn, style_id, vendor_id)?;
    conn.execute(
        "INSERT INTO sample (style_id, vendor_id, round, outcome) VALUES (?1, ?2, ?3, ?4)",
        params![style_id, vendor_id, round, SampleOutcome::Pending.as_db()],
    )?;
    get_sample(lib, conn.last_insert_rowid())
}

/// Reads one Sample.
pub fn get_sample(lib: &Library, id: SampleId) -> CoreResult<Sample> {
    let sample = lib
        .db()
        .query_row(&format!("SELECT {COLUMNS} FROM sample WHERE id = ?1"), [id], from_row)
        .optional()?
        .ok_or(CoreError::NotFound { record: "Sample", id })?;
    complete(lib, sample)
}

/// Every Sample of a Style, oldest first.
pub fn style_samples(lib: &Library, style_id: StyleId) -> CoreResult<Vec<Sample>> {
    style::get_style(lib, style_id)?;
    let mut stmt = lib.db().prepare(&format!("SELECT {COLUMNS} FROM sample WHERE style_id = ?1 ORDER BY id"))?;
    let rows: Vec<Sample> = stmt.query_map([style_id], from_row)?.collect::<Result<_, _>>()?;
    rows.into_iter().map(|s| complete(lib, s)).collect()
}

/// The round number the next Sample for this Style and Vendor gets.
fn next_round(conn: &Connection, style_id: StyleId, vendor_id: VendorId) -> CoreResult<u32> {
    Ok(conn.query_row(
        "SELECT COALESCE(MAX(round), 0) + 1 FROM sample WHERE style_id = ?1 AND vendor_id = ?2",
        [style_id, vendor_id],
        |r| r.get(0),
    )?)
}

/// Replaces a Sample's feedback.
pub fn set_sample_feedback(lib: &Library, id: SampleId, feedback: &str) -> CoreResult<Sample> {
    get_sample(lib, id)?;
    lib.writer()?.execute("UPDATE sample SET feedback = ?1 WHERE id = ?2", params![feedback.trim(), id])?;
    get_sample(lib, id)
}

/// Decides a Pending Sample: approved, revised, or rejected. Marking it
/// revised also creates the next round for the same Style and Vendor,
/// Pending, with this Sample's feedback copied forward. Returns the decided
/// Sample. Refuses a Sample that is already decided, and Pending as a
/// decision.
pub fn decide_sample(lib: &Library, id: SampleId, outcome: SampleOutcome) -> CoreResult<Sample> {
    let sample = get_sample(lib, id)?;
    if sample.outcome != SampleOutcome::Pending || outcome == SampleOutcome::Pending {
        return Err(CoreError::StatusMoveRefused { what: "Sample", from: sample.outcome.label(), to: outcome.label() });
    }
    let conn = lib.writer()?;
    let tx = conn.unchecked_transaction()?;
    tx.execute("UPDATE sample SET outcome = ?1 WHERE id = ?2", params![outcome.as_db(), id])?;
    if outcome == SampleOutcome::Revised {
        let round = next_round(&tx, sample.style_id, sample.vendor_id)?;
        tx.execute(
            "INSERT INTO sample (style_id, vendor_id, round, outcome, feedback) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![sample.style_id, sample.vendor_id, round, SampleOutcome::Pending.as_db(), sample.feedback],
        )?;
    }
    tx.commit()?;
    get_sample(lib, id)
}

/// Attaches photos to a Sample through the media import path, and returns
/// all its photos. `sources` are files outside the library; the core stores
/// a copy of each.
pub fn attach_sample_photos(lib: &Library, id: SampleId, sources: &[PathBuf]) -> CoreResult<Vec<SamplePhoto>> {
    get_sample(lib, id)?;
    let conn = lib.writer()?;
    let imported: Vec<(MediaRef, String)> = sources
        .iter()
        .map(|source| Ok((media::import_media(lib, source)?, display_name(source))))
        .collect::<CoreResult<_>>()?;
    let tx = conn.unchecked_transaction()?;
    for (media, file_name) in &imported {
        tx.execute(
            "INSERT INTO sample_photo (sample_id, media, file_name) VALUES (?1, ?2, ?3)",
            params![id, media.file_name(), file_name],
        )?;
    }
    tx.commit()?;
    read_photos(lib, id)
}

/// True when a Sample with this ID exists.
pub(crate) fn sample_exists(conn: &Connection, id: SampleId) -> CoreResult<bool> {
    Ok(conn.query_row("SELECT 1 FROM sample WHERE id = ?1", [id], |_| Ok(())).optional()?.is_some())
}

fn complete(lib: &Library, mut sample: Sample) -> CoreResult<Sample> {
    sample.photos = read_photos(lib, sample.id)?;
    Ok(sample)
}

fn read_photos(lib: &Library, id: SampleId) -> CoreResult<Vec<SamplePhoto>> {
    let mut stmt = lib.db().prepare("SELECT id, media, file_name FROM sample_photo WHERE sample_id = ?1 ORDER BY id")?;
    let rows: Vec<(i64, String, String)> =
        stmt.query_map([id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?.collect::<Result<_, _>>()?;
    Ok(rows
        .into_iter()
        .map(|(photo_id, stored, file_name)| {
            let media = MediaRef::from_stored(stored);
            let path = media::media_path(lib, &media);
            SamplePhoto { id: photo_id, media, file_name, path }
        })
        .collect())
}

fn display_name(source: &Path) -> String {
    source.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| source.display().to_string())
}
