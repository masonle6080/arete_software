//! Media: the one import path for files (images, PDFs, video) that other
//! areas (Moodboards, Styles, Clips, Notes) attach.
//!
//! Import always copies the file into `<library>/media/` (ADR-0002). The copy
//! is named by the BLAKE3 hash of its content, so importing the same content
//! twice stores one file and returns the same [`MediaRef`].
//!
//! Domain command functions for this area: [`import_media`], [`media_path`].

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::CoreResult;
use crate::library::Library;

/// A reference to one stored media file: its file name in the media folder,
/// `<hash>` or `<hash>.<extension>`. Store this string in records.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct MediaRef(String);

impl MediaRef {
    /// The file name inside the media folder.
    pub fn file_name(&self) -> &str {
        &self.0
    }

    /// A reference read back from a record. Only areas that stored it call this.
    pub(crate) fn from_stored(file_name: String) -> MediaRef {
        MediaRef(file_name)
    }
}

/// The absolute path of a stored media file.
pub fn media_path(lib: &Library, media: &MediaRef) -> PathBuf {
    lib.media_dir().join(media.file_name())
}

/// Copies the file at `source` into the library and returns its reference.
/// Refuses with `LibraryDisconnected`, before any copy, when the library is
/// unreachable.
pub fn import_media(lib: &Library, source: &Path) -> CoreResult<MediaRef> {
    lib.ensure_reachable()?;
    let media_dir = lib.media_dir();
    let tmp = media_dir.join(".import.tmp");
    let hash = copy_and_hash(source, &tmp).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    if let Some(existing) = find_by_hash(&media_dir, &hash)? {
        std::fs::remove_file(&tmp)?;
        return Ok(existing);
    }
    let media = MediaRef(file_name(&hash, source));
    std::fs::rename(&tmp, media_path(lib, &media))?;
    Ok(media)
}

/// The stored file with this content hash, whatever its extension.
fn find_by_hash(media_dir: &Path, hash: &str) -> CoreResult<Option<MediaRef>> {
    for entry in std::fs::read_dir(media_dir)? {
        let name = entry?.file_name();
        let Some(name) = name.to_str() else { continue };
        if name == hash || name.strip_prefix(hash).is_some_and(|rest| rest.starts_with('.')) {
            return Ok(Some(MediaRef(name.to_string())));
        }
    }
    Ok(None)
}

/// Streams `source` into `dest` and returns the hex BLAKE3 hash of the bytes.
fn copy_and_hash(source: &Path, dest: &Path) -> CoreResult<String> {
    let mut input = std::fs::File::open(source)?;
    let mut output = std::fs::File::create(dest)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        output.write_all(&buf[..n])?;
    }
    output.sync_all()?;
    Ok(hasher.finalize().to_hex().to_string())
}

/// `<hash>.<ext>`, with the source's extension in lower case, or `<hash>`.
fn file_name(hash: &str, source: &Path) -> String {
    match source.extension().and_then(|e| e.to_str()) {
        Some(ext) if !ext.is_empty() => format!("{hash}.{}", ext.to_ascii_lowercase()),
        _ => hash.to_string(),
    }
}
