//! Per-machine app settings, outside any library. Today it holds only the
//! path of the last library, so the app can reopen it on launch.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CoreResult;

const FILE: &str = "app-config.json";

#[derive(Debug, Default, Serialize, Deserialize)]
struct Stored {
    last_library: Option<PathBuf>,
}

/// The app config folder. In the app this is Tauri's app config dir; in
/// tests it is a temporary folder.
#[derive(Debug, Clone)]
pub struct AppConfig {
    dir: PathBuf,
}

impl AppConfig {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        AppConfig { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn last_library(&self) -> CoreResult<Option<PathBuf>> {
        Ok(self.load()?.last_library)
    }

    pub fn remember_last_library(&self, path: &Path) -> CoreResult<()> {
        let mut stored = self.load()?;
        stored.last_library = Some(path.to_path_buf());
        self.save(&stored)
    }

    fn load(&self) -> CoreResult<Stored> {
        match std::fs::read_to_string(self.dir.join(FILE)) {
            // A damaged config file is not worth failing launch over.
            Ok(text) => Ok(serde_json::from_str(&text).unwrap_or_default()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Stored::default()),
            Err(e) => Err(e.into()),
        }
    }

    fn save(&self, stored: &Stored) -> CoreResult<()> {
        std::fs::create_dir_all(&self.dir)?;
        let text = serde_json::to_string_pretty(stored).expect("serialize app config");
        std::fs::write(self.dir.join(FILE), text)?;
        Ok(())
    }
}
