//! Clip pool: every video file in the library (GLOSSARY: Clip). A Clip can
//! exist before any Documentary, so the pool is library-wide.
//!
//! Import goes through the one media import path ([`media::import_media`]),
//! so the video is copied into the library and deduplicated by content. A
//! [`VideoProbe`] then reads the duration and date shot and makes a
//! thumbnail. The real probe is [`FfmpegProbe`]; tests can pass a fake.
//!
//! Domain command functions for this area: [`import_clip`], [`list_clips`],
//! [`get_clip`], [`set_date_shot`], and in [`links`] the tags, notes,
//! Passage links, and filters.

mod ffmpeg;
mod links;

use std::path::{Path, PathBuf};

use rusqlite::{params, OptionalExtension, Row};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;
use crate::media::{self, MediaRef};

pub use ffmpeg::FfmpegProbe;
pub use links::*;

/// What a probe reads from a video file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VideoFacts {
    pub duration_ms: Option<u64>,
    /// `YYYY-MM-DD`, when the file records when it was shot.
    pub date_shot: Option<String>,
}

/// The seam over ffmpeg. `probe` reads the facts of `video` and writes a JPEG
/// thumbnail to `thumbnail`. When it cannot make a thumbnail it leaves
/// `thumbnail` absent and still returns the facts.
pub trait VideoProbe {
    fn probe(&self, video: &Path, thumbnail: &Path) -> CoreResult<VideoFacts>;
}

/// One Clip in the pool.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: i64,
    pub media: MediaRef,
    pub original_name: String,
    pub duration_ms: Option<u64>,
    /// `YYYY-MM-DD`, or `None` when unknown.
    pub date_shot: Option<String>,
    /// The video file inside the library.
    pub path: PathBuf,
    /// The JPEG thumbnail inside the library, if one was made.
    pub thumbnail_path: Option<PathBuf>,
    /// Tags, in alphabetical order (case ignored).
    pub tags: Vec<String>,
    /// Free-text notes; empty when none.
    pub notes: String,
}

const COLUMNS: &str = "id, media, original_name, duration_ms, date_shot, thumbnail, notes";

/// Imports the video at `source` into the Clip pool. Importing the same
/// content again returns the existing Clip unchanged.
pub fn import_clip(lib: &Library, source: &Path, probe: &dyn VideoProbe) -> CoreResult<Clip> {
    lib.ensure_reachable()?;
    let stored = probe_source(lib, source, probe)?;
    let video = media::import_media(lib, source)?;
    if let Some(existing) = find_by_media(lib, &video)? {
        return Ok(existing);
    }
    let thumbnail = match &stored.thumbnail {
        Some(tmp) => {
            let thumb = media::import_media(lib, tmp);
            let _ = std::fs::remove_file(tmp);
            Some(thumb?)
        }
        None => None,
    };
    let original_name = source
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    lib.writer()?.execute(
        "INSERT INTO clip (media, original_name, duration_ms, date_shot, thumbnail)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            video.file_name(),
            original_name,
            stored.facts.duration_ms.map(|d| d as i64),
            stored.facts.date_shot,
            thumbnail.as_ref().map(|t| t.file_name()),
        ],
    )?;
    get_clip(lib, lib.db().last_insert_rowid())
}

struct Probed {
    facts: VideoFacts,
    /// A temporary thumbnail file, when the probe made one.
    thumbnail: Option<PathBuf>,
}

/// Probes `source` before anything is stored, so a file that is not video
/// leaves the library unchanged.
fn probe_source(lib: &Library, source: &Path, probe: &dyn VideoProbe) -> CoreResult<Probed> {
    let tmp = lib.media_dir().join(".thumbnail.tmp.jpg");
    let _ = std::fs::remove_file(&tmp);
    let facts = probe.probe(source, &tmp).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    let date_shot = facts.date_shot.filter(|d| is_valid_date(d));
    let thumbnail = tmp.is_file().then_some(tmp);
    Ok(Probed { facts: VideoFacts { date_shot, ..facts }, thumbnail })
}

/// Every Clip in the pool, newest date shot first; Clips with no date last.
pub fn list_clips(lib: &Library) -> CoreResult<Vec<Clip>> {
    let mut stmt = lib.db().prepare(&format!(
        "SELECT {COLUMNS} FROM clip ORDER BY date_shot IS NULL, date_shot DESC, id DESC"
    ))?;
    let clips = stmt.query_map([], |r| from_row(lib, r))?.collect::<Result<_, _>>()?;
    Ok(clips)
}

/// The Clip with this id.
pub fn get_clip(lib: &Library, id: i64) -> CoreResult<Clip> {
    lib.db()
        .query_row(&format!("SELECT {COLUMNS} FROM clip WHERE id = ?1"), [id], |r| from_row(lib, r))
        .optional()?
        .ok_or(CoreError::NotFound { record: "Clip", id })
}

/// Sets or clears (`None`) the date a Clip was shot. The date is `YYYY-MM-DD`.
pub fn set_date_shot(lib: &Library, id: i64, date_shot: Option<&str>) -> CoreResult<Clip> {
    if let Some(d) = date_shot {
        if !is_valid_date(d) {
            return Err(CoreError::InvalidInput { reason: format!("{d:?} is not a date; use YYYY-MM-DD") });
        }
    }
    let changed =
        lib.writer()?.execute("UPDATE clip SET date_shot = ?1 WHERE id = ?2", params![date_shot, id])?;
    if changed == 0 {
        return Err(CoreError::NotFound { record: "Clip", id });
    }
    get_clip(lib, id)
}

fn find_by_media(lib: &Library, video: &MediaRef) -> CoreResult<Option<Clip>> {
    Ok(lib
        .db()
        .query_row(
            &format!("SELECT {COLUMNS} FROM clip WHERE media = ?1"),
            [video.file_name()],
            |r| from_row(lib, r),
        )
        .optional()?)
}

fn from_row(lib: &Library, r: &Row) -> rusqlite::Result<Clip> {
    let video = MediaRef::from_stored(r.get::<_, String>(1)?);
    let thumbnail: Option<String> = r.get(5)?;
    let id: i64 = r.get(0)?;
    Ok(Clip {
        id,
        path: media::media_path(lib, &video),
        media: video,
        original_name: r.get(2)?,
        duration_ms: r.get::<_, Option<i64>>(3)?.map(|d| d as u64),
        date_shot: r.get(4)?,
        thumbnail_path: thumbnail.map(|t| media::media_path(lib, &MediaRef::from_stored(t))),
        tags: links::tags_of(lib, id)?,
        notes: r.get(6)?,
    })
}

/// `YYYY-MM-DD` with a real month and day.
fn is_valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let num = |r: std::ops::Range<usize>| {
        let part = &s[r];
        part.bytes().all(|c| c.is_ascii_digit()).then(|| part.parse::<u32>().ok()).flatten()
    };
    let (Some(y), Some(m), Some(d)) = (num(0..4), num(5..7), num(8..10)) else {
        return false;
    };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&d)
}
