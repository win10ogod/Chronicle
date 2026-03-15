use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct ProjectPaths {
    root: PathBuf,
}

impl ProjectPaths {
    pub fn discover(root_override: Option<&Path>) -> Result<Self> {
        let start = match root_override {
            Some(p) => p.to_path_buf(),
            None => std::env::current_dir().context("get current working directory")?,
        };

        let root = Self::find_project_root(&start)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn chronicle_dir(&self) -> PathBuf {
        self.root.join(".chronicle")
    }

    pub fn short_term_dir(&self) -> PathBuf {
        self.chronicle_dir().join("short_term")
    }

    pub fn long_term_dir(&self) -> PathBuf {
        self.chronicle_dir().join("long_term")
    }

    pub fn archive_dir(&self) -> PathBuf {
        self.chronicle_dir().join("archive")
    }

    pub fn index_file(&self) -> PathBuf {
        self.chronicle_dir().join("index.json")
    }

    pub fn index_lock_file(&self) -> PathBuf {
        self.chronicle_dir().join("index.lock")
    }

    pub fn wal_dir(&self) -> PathBuf {
        self.chronicle_dir().join("wal")
    }

    pub fn wal_lock_file(&self) -> PathBuf {
        self.chronicle_dir().join("wal.lock")
    }

    fn find_project_root(start: &Path) -> Result<PathBuf> {
        for dir in start.ancestors() {
            if dir.join(".chronicle").is_dir() {
                return Ok(dir.to_path_buf());
            }
        }

        for dir in start.ancestors() {
            if dir.join(".git").is_dir() || dir.join(".git").is_file() {
                return Ok(dir.to_path_buf());
            }
        }

        Ok(start.to_path_buf())
    }
}
