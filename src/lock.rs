use std::fs::OpenOptions;
use std::path::Path;

use anyhow::{Context, Result};

pub fn with_write_lock<T>(lock_path: &Path, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .with_context(|| format!("open lock file at {}", lock_path.display()))?;

    let mut lock = fd_lock::RwLock::new(file);
    let _guard = lock
        .write()
        .with_context(|| format!("acquire write lock at {}", lock_path.display()))?;

    f()
}

pub fn with_read_lock<T>(lock_path: &Path, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .with_context(|| format!("open lock file at {}", lock_path.display()))?;

    let lock = fd_lock::RwLock::new(file);
    let _guard = lock
        .read()
        .with_context(|| format!("acquire read lock at {}", lock_path.display()))?;

    f()
}
