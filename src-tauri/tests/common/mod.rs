//! Shared test support for Library core integration tests.
//!
//! Use `TestEnv::new()` for a fresh temporary app config and a scratch folder,
//! and `TestEnv::library()` when a test only needs an open library.
#![allow(dead_code)]

use std::path::PathBuf;

use arete_lib::library::{self, AppConfig, Library};
use arete_lib::thesis::{self, Thesis, ThesisId, ThesisMove, ThesisStatus};
use tempfile::TempDir;

pub struct TestEnv {
    _tmp: TempDir,
    root: PathBuf,
    pub config: AppConfig,
}

impl TestEnv {
    pub fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().to_path_buf();
        let config = AppConfig::new(root.join("app-config"));
        TestEnv { _tmp: tmp, root, config }
    }

    /// A path inside the scratch folder. The path does not exist yet.
    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join("scratch").join(rel)
    }

    /// A fresh, open library at the latest schema version.
    pub fn library(&self) -> Library {
        library::create_library(&self.config, &self.path("library")).unwrap()
    }
}

/// Moves Thesis `id` to `to` with `confirmed` set, and expects the move to
/// go through.
pub fn move_to(lib: &Library, id: ThesisId, to: ThesisStatus) -> Thesis {
    match thesis::move_thesis(lib, id, to, true).unwrap() {
        ThesisMove::Moved(t) => t,
        other => panic!("→ {to}: {other:?}"),
    }
}

/// Moves a new Thesis along the lifecycle to `at`, or straight to
/// Abandoned. The Thesis must have no Style that blocks Production.
pub fn walk_to(lib: &Library, id: i64, at: ThesisStatus) {
    use ThesisStatus::*;
    if at == Abandoned {
        move_to(lib, id, Abandoned);
        return;
    }
    for step in [Developing, Design, Sampling, Production, Released, Archived] {
        if thesis::get_thesis(lib, id).unwrap().status == at {
            return;
        }
        move_to(lib, id, step);
    }
}

/// A new Thesis moved to Design, so it takes new Styles.
pub fn design_thesis(lib: &Library, title: &str) -> Thesis {
    let t = thesis::create_thesis(lib, title, "").unwrap();
    move_to(lib, t.id, ThesisStatus::Developing);
    move_to(lib, t.id, ThesisStatus::Design)
}
