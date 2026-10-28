//! Media: the one import path for files (images, PDFs, video) that other
//! areas (Moodboards, Styles, Clips, Notes) attach.
//!
//! Import always copies the file into `<library>/media/` (ADR-0002). The copy
//! is named by the BLAKE3 hash of its content, so importing the same content
//! twice stores one file and returns the same [`MediaRef`].
//!
//! Domain command functions for this area: [`import_media`],
//! [`import_media_bytes`], [`read_media`], [`media_path`].

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{CoreError, CoreResult};
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

    /// Rebuilds a reference that a record stored with [`MediaRef::file_name`].
    pub(crate) fn from_stored(file_name: String) -> MediaRef {
        MediaRef(file_name)
    }
}

/// The absolute path of a stored media file.
pub fn media_path(lib: &Library, media: &MediaRef) -> PathBuf {
    lib.media_dir().join(media.file_name())
}

/// The bytes of a stored media file. Refuses with `InvalidInput` a reference
/// that is not a plain media file name (a `MediaRef` can arrive over IPC).
pub fn read_media(lib: &Library, media: &MediaRef) -> CoreResult<Vec<u8>> {
    let name = media.file_name();
    let plain = !name.is_empty()
        && !name.starts_with('.')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '.');
    if !plain {
        return Err(CoreError::InvalidInput { reason: format!("{name:?} is not a media file name") });
    }
    Ok(std::fs::read(media_path(lib, media))?)
}

/// Copies the file at `source` into the library and returns its reference.
/// Refuses with `LibraryDisconnected`, before any copy, when the library is
/// unreachable.
pub fn import_media(lib: &Library, source: &Path) -> CoreResult<MediaRef> {
    lib.ensure_reachable()?;
    let input = std::fs::File::open(source)?;
    store(lib, input, source)
}

/// Stores `bytes` in the library and returns its reference, with the same
/// dedup as [`import_media`]. Use it for content that has no file path, for
/// example an image dragged from a browser. `name` is the original file
/// name; only its extension is kept.
pub fn import_media_bytes(lib: &Library, bytes: &[u8], name: &str) -> CoreResult<MediaRef> {
    lib.ensure_reachable()?;
    store(lib, bytes, Path::new(name))
}

/// Copies `input` into the media folder under its content hash. `named` gives
/// the extension.
fn store(lib: &Library, input: impl Read, named: &Path) -> CoreResult<MediaRef> {
    let media_dir = lib.media_dir();
    let tmp = media_dir.join(".import.tmp");
    let hash = copy_and_hash(input, &tmp).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })?;
    if let Some(existing) = find_by_hash(&media_dir, &hash)? {
        std::fs::remove_file(&tmp)?;
        return Ok(existing);
    }
    let media = MediaRef(file_name(&hash, named));
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

/// Streams `input` into `dest` and returns the hex BLAKE3 hash of the bytes.
fn copy_and_hash(mut input: impl Read, dest: &Path) -> CoreResult<String> {
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
