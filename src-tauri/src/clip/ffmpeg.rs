//! The real [`VideoProbe`]: ffprobe reads the facts, ffmpeg makes the
//! thumbnail. Both run locally as child processes; nothing uses the network.
//!
//! The release bundle ships both as Tauri sidecars (`bundle.externalBin`),
//! which Tauri places next to the app executable. See the README.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use super::{VideoFacts, VideoProbe};
use crate::error::{CoreError, CoreResult};

/// Runs the ffmpeg and ffprobe executables at fixed paths.
#[derive(Debug, Clone)]
pub struct FfmpegProbe {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
}

impl FfmpegProbe {
    pub fn new(ffmpeg: PathBuf, ffprobe: PathBuf) -> Self {
        FfmpegProbe { ffmpeg, ffprobe }
    }

    /// Finds ffmpeg and ffprobe, first match wins:
    /// 1. `ARETE_FFMPEG` and `ARETE_FFPROBE`
    /// 2. the bundled sidecars next to the app executable
    /// 3. `/opt/homebrew/bin`, `/usr/local/bin`, then `PATH` (development)
    ///
    /// Fails with `VideoToolMissing` when either one is not found.
    pub fn locate() -> CoreResult<FfmpegProbe> {
        Ok(FfmpegProbe::new(find_tool("ffmpeg", "ARETE_FFMPEG")?, find_tool("ffprobe", "ARETE_FFPROBE")?))
    }
}

fn find_tool(name: &str, env_var: &str) -> CoreResult<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(p) = std::env::var_os(env_var) {
        candidates.push(PathBuf::from(p));
    }
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        candidates.push(dir.join(name));
    }
    candidates.push(Path::new("/opt/homebrew/bin").join(name));
    candidates.push(Path::new("/usr/local/bin").join(name));
    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&path).map(|d| d.join(name)));
    }
    candidates.into_iter().find(|p| p.is_file()).ok_or(CoreError::VideoToolMissing)
}

impl VideoProbe for FfmpegProbe {
    fn probe(&self, video: &Path, thumbnail: &Path) -> CoreResult<VideoFacts> {
        let info = self.read_info(video)?;
        let facts = facts_from_info(&info);
        if !has_video_stream(&info) {
            return Err(CoreError::NotAVideo { path: video.to_path_buf() });
        }
        // A frame one second in avoids black first frames; short clips use 0.
        let at = if facts.duration_ms.is_some_and(|d| d > 2_000) { "1" } else { "0" };
        let made = self.write_thumbnail(video, thumbnail, at)? || self.write_thumbnail(video, thumbnail, "0")?;
        if !made {
            let _ = std::fs::remove_file(thumbnail);
        }
        Ok(facts)
    }
}

impl FfmpegProbe {
    fn read_info(&self, video: &Path) -> CoreResult<Value> {
        let out = Command::new(&self.ffprobe)
            .args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams"])
            .arg(video)
            .stdin(Stdio::null())
            .output()
            .map_err(|_| CoreError::VideoToolMissing)?;
        if !out.status.success() {
            return Err(CoreError::NotAVideo { path: video.to_path_buf() });
        }
        serde_json::from_slice(&out.stdout).map_err(|_| CoreError::NotAVideo { path: video.to_path_buf() })
    }

    /// True when ffmpeg wrote a non-empty JPEG to `thumbnail`.
    fn write_thumbnail(&self, video: &Path, thumbnail: &Path, at: &str) -> CoreResult<bool> {
        let status = Command::new(&self.ffmpeg)
            .args(["-v", "error", "-y", "-ss", at, "-i"])
            .arg(video)
            .args(["-frames:v", "1", "-vf", "scale=480:-2", "-q:v", "4", "-f", "image2", "-c:v", "mjpeg"])
            .arg(thumbnail)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| CoreError::VideoToolMissing)?;
        Ok(status.success() && std::fs::metadata(thumbnail).is_ok_and(|m| m.len() > 0))
    }
}

fn has_video_stream(info: &Value) -> bool {
    info["streams"].as_array().is_some_and(|streams| {
        streams.iter().any(|s| {
            s["codec_type"] == "video" && s["disposition"]["attached_pic"].as_i64().unwrap_or(0) == 0
        })
    })
}

fn facts_from_info(info: &Value) -> VideoFacts {
    let duration_ms = info["format"]["duration"]
        .as_str()
        .and_then(|d| d.parse::<f64>().ok())
        .filter(|d| d.is_finite() && *d >= 0.0)
        .map(|d| (d * 1000.0).round() as u64);

    // Apple devices record the local shooting time; `creation_time` is UTC.
    let format_tags = &info["format"]["tags"];
    let stream_tags = info["streams"].as_array().into_iter().flatten().map(|s| &s["tags"]);
    let date_shot = std::iter::once(&format_tags["com.apple.quicktime.creationdate"])
        .chain(std::iter::once(&format_tags["creation_time"]))
        .chain(stream_tags.map(|t| &t["creation_time"]))
        .filter_map(Value::as_str)
        .find_map(|s| s.get(..10).map(str::to_string));

    VideoFacts { duration_ms, date_shot }
}
