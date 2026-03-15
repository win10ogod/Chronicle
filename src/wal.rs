use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::git::Git;
use crate::lock;
use crate::paths::ProjectPaths;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
pub enum CommitMode {
    Sync,
    Async,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalTask {
    version: u8,
    created_at: DateTime<Utc>,
    message: String,
    pathspecs: Vec<String>,
}

pub fn enqueue(paths: &ProjectPaths, message: String, staged_paths: &[PathBuf]) -> Result<PathBuf> {
    fs::create_dir_all(paths.wal_dir())
        .with_context(|| format!("create {}", paths.wal_dir().display()))?;

    let created_at = Utc::now();
    let rand_suffix: u16 = rand::random();
    let file_name = format!(
        "task_{}_{}.json",
        created_at.format("%Y%m%d_%H%M%S"),
        rand_suffix
    );
    let task_path = paths.wal_dir().join(file_name);

    let mut specs = Vec::new();
    for p in staged_paths {
        let spec = p
            .strip_prefix(paths.root())
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        if spec.is_empty() {
            continue;
        }
        specs.push(spec);
    }
    if specs.is_empty() {
        specs.push(".chronicle".to_string());
    }

    let task = WalTask {
        version: 1,
        created_at,
        message,
        pathspecs: specs,
    };

    let bytes = serde_json::to_vec_pretty(&task).context("serialize WAL task")?;
    atomic_write(&task_path, &bytes)?;
    Ok(task_path)
}

pub fn run_once(paths: &ProjectPaths) -> Result<usize> {
    lock::with_write_lock(&paths.wal_lock_file(), || run_once_locked(paths))
}

fn run_once_locked(paths: &ProjectPaths) -> Result<usize> {
    if !paths.wal_dir().is_dir() {
        return Ok(0);
    }

    let mut tasks = fs::read_dir(paths.wal_dir())
        .with_context(|| format!("read {}", paths.wal_dir().display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    tasks.sort();
    if tasks.is_empty() {
        return Ok(0);
    }

    let mut git = Git::open(paths.root())?;
    let mut processed = 0usize;

    for task_path in tasks {
        let bytes = fs::read(&task_path)
            .with_context(|| format!("read WAL task {}", task_path.display()))?;
        let task: WalTask = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse WAL task {}", task_path.display()))?;
        if task.version != 1 {
            return Err(anyhow!("unsupported WAL task version {}", task.version));
        }

        let specs: Vec<PathBuf> = task
            .pathspecs
            .iter()
            .map(|s| paths.root().join(s))
            .collect();

        git.stage_paths(&specs)?;
        git.commit(&task.message)?;

        fs::remove_file(&task_path)
            .with_context(|| format!("remove WAL task {}", task_path.display()))?;
        processed += 1;
    }

    Ok(processed)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("create temp file in {}", parent.display()))?;
    use std::io::Write as _;
    tmp.write_all(bytes)
        .with_context(|| format!("write temp file for {}", path.display()))?;
    tmp.flush().context("flush temp file")?;
    tmp.as_file().sync_all().context("fsync temp file")?;

    match tmp.persist(path) {
        Ok(_) => Ok(()),
        Err(err) => {
            let (tmp, persist_err) = (err.file, err.error);
            if persist_err.kind() == std::io::ErrorKind::AlreadyExists {
                fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
                tmp.persist(path)
                    .map(|_| ())
                    .with_context(|| format!("persist {}", path.display()))
            } else {
                Err(persist_err).with_context(|| format!("persist {}", path.display()))
            }
        }
    }
}
