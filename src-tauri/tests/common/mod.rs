//! Shared test support for Library core integration tests.
//!
//! Use `TestEnv::new()` for a fresh temporary app config and a scratch folder,
//! and `TestEnv::library()` when a test only needs an open library.
#![allow(dead_code)]

use std::path::PathBuf;

use arete_lib::library::{self, AppConfig, Library};
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
