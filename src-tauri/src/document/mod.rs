//! Document: a long-form text the founder writes in the app (ADR-0001).
//!
//! Essays use it now; Scripts will use it too. Storage is the same for every
//! [`DocumentKind`]: the text is one Markdown file in the library's
//! `documents/` folder, and the `document` table holds the record (id, kind,
//! title, file name, timestamps). The core stores the Markdown exactly as the
//! webview sends it. Parsing it (Passage markers included) is the job of the
//! TypeScript document module (`src/document/`).
//!
//! Domain command functions for this area: [`create_document`],
//! [`open_document`], [`save_document`], [`list_documents`].

use std::path::PathBuf;

use rusqlite::{params, OptionalExtension, Row};

use crate::error::{CoreError, CoreResult};
use crate::library::Library;

/// What a document is for. Storage does not depend on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocumentKind {
    Essay,
    Script,
}

impl DocumentKind {
    fn as_str(self) -> &'static str {
        match self {
            DocumentKind::Essay => "essay",
            DocumentKind::Script => "script",
        }
    }

    fn parse(s: &str) -> rusqlite::Result<Self> {
        match s {
            "essay" => Ok(DocumentKind::Essay),
            "script" => Ok(DocumentKind::Script),
            other => Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                format!("unknown document kind {other:?}").into(),
            )),
        }
    }
}

/// A document's record, without its text. Lists show these.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    pub id: i64,
    pub kind: DocumentKind,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A document with its Markdown text, as the editor opens it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: i64,
    pub kind: DocumentKind,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub markdown: String,
}

/// What a save changes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentEdit {
    pub title: String,
    pub markdown: String,
}

const COLUMNS: &str = "id, kind, title, created_at, updated_at, file_name";

struct Record {
    summary: DocumentSummary,
    file_name: String,
}

fn record(row: &Row) -> rusqlite::Result<Record> {
    let kind: String = row.get(1)?;
    Ok(Record {
        summary: DocumentSummary {
            id: row.get(0)?,
            kind: DocumentKind::parse(&kind)?,
            title: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        },
        file_name: row.get(5)?,
    })
}

fn find(lib: &Library, id: i64) -> CoreResult<Record> {
    lib.db()
        .query_row(&format!("SELECT {COLUMNS} FROM document WHERE id = ?1"), [id], record)
        .optional()?
        .ok_or(CoreError::NotFound { record: "Document", id })
}

fn file_path(lib: &Library, file_name: &str) -> PathBuf {
    lib.documents_dir().join(file_name)
}

/// Writes `text` to `path` through a temporary file and a rename, so a crash
/// never leaves a half-written document.
fn write_atomically(path: &std::path::Path, text: &str) -> CoreResult<()> {
    let tmp = path.with_extension("md.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn with_text(summary: DocumentSummary, markdown: String) -> Document {
    let DocumentSummary { id, kind, title, created_at, updated_at } = summary;
    Document { id, kind, title, created_at, updated_at, markdown }
}

/// Creates an empty document of `kind`. It belongs to no Thesis.
pub fn create_document(lib: &Library, kind: DocumentKind, title: &str) -> CoreResult<Document> {
    let db = lib.writer()?;
    // A random file name stays unique even if a row id is ever reused.
    let stem: String = db.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))?;
    let file_name = format!("{stem}.md");
    write_atomically(&file_path(lib, &file_name), "")?;
    db.execute(
        "INSERT INTO document (kind, title, file_name, edit_seq)
         VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(edit_seq), 0) + 1 FROM document))",
        params![kind.as_str(), title, file_name],
    )?;
    open_document(lib, db.last_insert_rowid())
}

/// Opens a document with its Markdown text, exactly as last saved.
pub fn open_document(lib: &Library, id: i64) -> CoreResult<Document> {
    let rec = find(lib, id)?;
    let markdown = std::fs::read_to_string(file_path(lib, &rec.file_name))?;
    Ok(with_text(rec.summary, markdown))
}

/// Saves a document's title and Markdown text. The file holds exactly
/// `edit.markdown`, so the document reloads unchanged.
pub fn save_document(lib: &Library, id: i64, edit: &DocumentEdit) -> CoreResult<DocumentSummary> {
    let db = lib.writer()?;
    let rec = find(lib, id)?;
    write_atomically(&file_path(lib, &rec.file_name), &edit.markdown)?;
    db.execute(
        "UPDATE document
         SET title = ?2,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
             edit_seq = (SELECT COALESCE(MAX(edit_seq), 0) + 1 FROM document)
         WHERE id = ?1",
        params![id, edit.title],
    )?;
    Ok(find(lib, id)?.summary)
}

/// Lists the documents of `kind`, most recently edited first.
pub fn list_documents(lib: &Library, kind: DocumentKind) -> CoreResult<Vec<DocumentSummary>> {
    let mut stmt = lib.db().prepare(&format!(
        "SELECT {COLUMNS} FROM document WHERE kind = ?1 ORDER BY edit_seq DESC"
    ))?;
    let rows = stmt.query_map([kind.as_str()], record)?;
    Ok(rows.map(|r| r.map(|rec| rec.summary)).collect::<Result<_, _>>()?)
}
