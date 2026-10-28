//! Library core: the Clip pool.
//!
//! Most tests use a fake `VideoProbe`, so they run without ffmpeg. The tests
//! at the end use the real ffmpeg probe and skip when ffmpeg is not found.

mod common;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use arete_lib::clip::{self, FfmpegProbe, VideoFacts, VideoProbe};
use arete_lib::error::{CoreError, CoreResult};
use arete_lib::media;
use common::TestEnv;

/// A probe that reports fixed facts and writes fixed thumbnail bytes.
struct FakeProbe {
    facts: VideoFacts,
    thumbnail: &'static [u8],
    calls: AtomicUsize,
}

impl FakeProbe {
    fn new(duration_ms: Option<u64>, date_shot: Option<&str>) -> Self {
        FakeProbe {
            facts: VideoFacts { duration_ms, date_shot: date_shot.map(String::from) },
            thumbnail: b"fake jpeg",
            calls: AtomicUsize::new(0),
        }
    }
}

impl VideoProbe for FakeProbe {
    fn probe(&self, _video: &Path, thumbnail: &Path) -> CoreResult<VideoFacts> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        std::fs::write(thumbnail, self.thumbnail)?;
        Ok(self.facts.clone())
    }
}

/// A probe that refuses the file, like ffprobe on a file that is not video.
struct RefusingProbe;

impl VideoProbe for RefusingProbe {
    fn probe(&self, video: &Path, _thumbnail: &Path) -> CoreResult<VideoFacts> {
        Err(CoreError::NotAVideo { path: video.to_path_buf() })
    }
}

fn source_file(env: &TestEnv, name: &str, bytes: &[u8]) -> PathBuf {
    let path = env.path(&format!("outside/{name}"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn an_imported_clip_has_its_thumbnail_duration_and_date_shot() {
    let env = TestEnv::new();
    let lib = env.library();
    let src = source_file(&env, "Beach Walk.MOV", b"video bytes");
    let probe = FakeProbe::new(Some(12_500), Some("2026-05-01"));

    let clip = clip::import_clip(&lib, &src, &probe).unwrap();

    assert_eq!(clip.original_name, "Beach Walk.MOV");
    assert_eq!(clip.duration_ms, Some(12_500));
    assert_eq!(clip.date_shot.as_deref(), Some("2026-05-01"));
    assert_eq!(std::fs::read(&clip.path).unwrap(), b"video bytes");
    let thumb = clip.thumbnail_path.as_ref().expect("a thumbnail");
    assert_eq!(std::fs::read(thumb).unwrap(), b"fake jpeg");
    assert_eq!(clip::list_clips(&lib).unwrap(), vec![clip]);
}

#[test]
fn importing_the_same_video_again_returns_the_existing_clip() {
    let env = TestEnv::new();
    let lib = env.library();
    let first = source_file(&env, "a/take.mov", b"same video");
    let again = source_file(&env, "b/take copy.mov", b"same video");
    let probe = FakeProbe::new(Some(1_000), None);

    let a = clip::import_clip(&lib, &first, &probe).unwrap();
    let b = clip::import_clip(&lib, &again, &probe).unwrap();

    assert_eq!(a, b);
    assert_eq!(clip::list_clips(&lib).unwrap().len(), 1);
}

#[test]
fn a_file_that_is_not_video_is_refused_and_nothing_is_stored() {
    let env = TestEnv::new();
    let lib = env.library();
    let src = source_file(&env, "notes.txt", b"not a video");

    let err = clip::import_clip(&lib, &src, &RefusingProbe).unwrap_err();

    assert!(matches!(err, CoreError::NotAVideo { .. }), "{err:?}");
    assert!(clip::list_clips(&lib).unwrap().is_empty());
    assert_eq!(std::fs::read_dir(lib.media_dir()).unwrap().count(), 0);
}

#[test]
fn the_date_shot_can_be_edited_and_cleared() {
    let env = TestEnv::new();
    let lib = env.library();
    let src = source_file(&env, "take.mov", b"video");
    let id = clip::import_clip(&lib, &src, &FakeProbe::new(None, Some("2026-05-01"))).unwrap().id;

    let edited = clip::set_date_shot(&lib, id, Some("2024-02-29")).unwrap();
    assert_eq!(edited.date_shot.as_deref(), Some("2024-02-29"));
    assert_eq!(clip::get_clip(&lib, id).unwrap().date_shot.as_deref(), Some("2024-02-29"));

    let cleared = clip::set_date_shot(&lib, id, None).unwrap();
    assert_eq!(cleared.date_shot, None);
}

#[test]
fn an_invalid_date_shot_is_refused() {
    let env = TestEnv::new();
    let lib = env.library();
    let src = source_file(&env, "take.mov", b"video");
    let id = clip::import_clip(&lib, &src, &FakeProbe::new(None, Some("2026-05-01"))).unwrap().id;

    for bad in ["2023-02-29", "2026-13-01", "May 1", "2026-5-1"] {
        let err = clip::set_date_shot(&lib, id, Some(bad)).unwrap_err();
        assert!(matches!(err, CoreError::InvalidInput { .. }), "{bad}: {err:?}");
    }
    assert_eq!(clip::get_clip(&lib, id).unwrap().date_shot.as_deref(), Some("2026-05-01"));
}

#[test]
fn editing_an_unknown_clip_fails() {
    let env = TestEnv::new();
    let lib = env.library();

    let err = clip::set_date_shot(&lib, 42, Some("2026-05-01")).unwrap_err();

    assert!(matches!(err, CoreError::NotFound { record: "Clip", id: 42 }), "{err:?}");
}

#[test]
fn clips_list_newest_date_shot_first_and_undated_last() {
    let env = TestEnv::new();
    let lib = env.library();
    let import = |name: &str, date: Option<&str>| {
        let src = source_file(&env, name, name.as_bytes());
        clip::import_clip(&lib, &src, &FakeProbe::new(None, date)).unwrap()
    };
    import("old.mov", Some("2025-01-01"));
    import("undated.mov", None);
    import("new.mov", Some("2026-01-01"));

    let names: Vec<_> = clip::list_clips(&lib).unwrap().into_iter().map(|c| c.original_name).collect();

    assert_eq!(names, ["new.mov", "old.mov", "undated.mov"]);
}

#[test]
fn import_is_refused_when_the_library_is_disconnected() {
    let env = TestEnv::new();
    let lib = env.library();
    let src = source_file(&env, "take.mov", b"video");
    let probe = FakeProbe::new(None, None);
    std::fs::rename(lib.root(), env.path("unplugged")).unwrap();

    let err = clip::import_clip(&lib, &src, &probe).unwrap_err();

    assert!(matches!(err, CoreError::LibraryDisconnected { .. }), "{err:?}");
    assert_eq!(probe.calls.load(Ordering::SeqCst), 0, "nothing was probed");
}

// --- The real ffmpeg probe. These skip when ffmpeg is not installed. ---

fn ffmpeg_probe() -> Option<FfmpegProbe> {
    match FfmpegProbe::locate() {
        Ok(p) => Some(p),
        Err(CoreError::VideoToolMissing) => {
            eprintln!("skipped: ffmpeg not found");
            None
        }
        Err(e) => panic!("{e:?}"),
    }
}

/// Makes a 3-second test video recorded as shot on 2025-06-15.
fn real_video(env: &TestEnv) -> PathBuf {
    let path = env.path("outside/shot.mp4");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let ok = std::process::Command::new(which_ffmpeg())
        .args(["-v", "error", "-f", "lavfi", "-i", "testsrc=duration=3:size=320x240:rate=10"])
        .args(["-metadata", "creation_time=2025-06-15T10:00:00Z", "-pix_fmt", "yuv420p"])
        .arg(&path)
        .status()
        .unwrap()
        .success();
    assert!(ok, "ffmpeg could not make the test video");
    path
}

fn which_ffmpeg() -> PathBuf {
    ["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from("ffmpeg"))
}

#[test]
fn ffmpeg_reads_duration_and_date_shot_and_makes_a_jpeg_thumbnail() {
    let Some(probe) = ffmpeg_probe() else { return };
    let env = TestEnv::new();
    let lib = env.library();
    let src = real_video(&env);

    let clip = clip::import_clip(&lib, &src, &probe).unwrap();

    assert_eq!(clip.duration_ms, Some(3_000));
    assert_eq!(clip.date_shot.as_deref(), Some("2025-06-15"));
    let thumb = std::fs::read(clip.thumbnail_path.unwrap()).unwrap();
    assert_eq!(&thumb[..2], [0xFF, 0xD8], "a JPEG");
    assert!(clip.path.starts_with(lib.media_dir()));
    assert_eq!(clip.media, media::import_media(&lib, &src).unwrap(), "stored as media");
}

#[test]
fn ffmpeg_refuses_a_file_that_is_not_video() {
    let Some(probe) = ffmpeg_probe() else { return };
    let env = TestEnv::new();
    let lib = env.library();
    let src = source_file(&env, "notes.mov", b"just text, named like a video");

    let err = clip::import_clip(&lib, &src, &probe).unwrap_err();

    assert!(matches!(err, CoreError::NotAVideo { .. }), "{err:?}");
    assert_eq!(std::fs::read_dir(lib.media_dir()).unwrap().count(), 0);
}
