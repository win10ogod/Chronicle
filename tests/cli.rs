use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;
use base64::Engine as _;
use predicates::prelude::PredicateBooleanExt as _;
use predicates::str::contains;
use tempfile::TempDir;

fn read_single_md(dir: &Path) -> PathBuf {
    let mut files = fs::read_dir(dir)
        .unwrap_or_else(|_| panic!("read_dir failed: {}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    files.sort();
    assert_eq!(files.len(), 1, "expected 1 .md file in {}", dir.display());
    files.remove(0)
}

fn git_output(root: &Path, args: &[&str]) -> String {
    let out = StdCommand::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .expect("git output");
    assert!(out.status.success(), "git {:?} failed", args);
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn remember_and_recall_roundtrip() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args([
            "remember",
            "--msg",
            "Rust 併發鎖 [[Rust]] #bug-fix",
            "--tags",
            "concurrency",
        ])
        .assert()
        .success();

    // Drain WAL to make Git history deterministic (default commit mode is async).
    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let short_dir = tmp.path().join(".chronicle/short_term");
    let file = read_single_md(&short_dir);
    let raw = fs::read_to_string(&file).expect("read memory file");
    assert!(raw.contains("hit_count"));

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args([
            "recall",
            "--query",
            "Rust 併發鎖",
            "--top",
            "1",
            "--no-touch",
        ])
        .assert()
        .success()
        .stdout(contains("Rust 併發鎖"));

    let commits = git_output(tmp.path(), &["rev-list", "--count", "HEAD"]);
    let n: u32 = commits.trim().parse().expect("commit count");
    assert!(n >= 1);
}

#[test]
fn branch_create_and_current() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    // Create an initial commit so HEAD is defined and branches can be queried consistently.
    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["remember", "--msg", "seed"])
        .assert()
        .success();

    // Ensure the initial commit exists before branching (default commit mode is async).
    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let base = git_output(tmp.path(), &["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["branch", "create", "hypothesis-A"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["branch", "current"])
        .assert()
        .success()
        .stdout(contains("hypothesis-A"));

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["branch", "checkout", &base])
        .assert()
        .success();
}

#[test]
fn consolidate_moves_short_to_long() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["remember", "--msg", "temp note"])
        .assert()
        .success();

    // Drain WAL to ensure baseline commit exists.
    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let short_dir = tmp.path().join(".chronicle/short_term");
    let long_dir = tmp.path().join(".chronicle/long_term");
    let file = read_single_md(&short_dir);

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["consolidate", "--min-hits", "1"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    assert!(
        !file.exists(),
        "expected {} moved out of short_term",
        file.display()
    );
    let _ = read_single_md(&long_dir);

    let commits = git_output(tmp.path(), &["rev-list", "--count", "HEAD"]);
    let n: u32 = commits.trim().parse().expect("commit count");
    assert!(n >= 2);
}

#[test]
fn encryption_roundtrip_requires_key() {
    let tmp = TempDir::new().expect("tempdir");

    let key = [42u8; 32];
    let key_b64 = base64::engine::general_purpose::STANDARD.encode(key);

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .env("CHRONICLE_KEY", format!("base64:{key_b64}"))
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .env("CHRONICLE_KEY", format!("base64:{key_b64}"))
        .args(["remember", "--msg", "secret message [[Rust]]"])
        .assert()
        .success();

    let short_dir = tmp.path().join(".chronicle/short_term");
    let file = read_single_md(&short_dir);
    let bytes = fs::read(&file).expect("read encrypted file");
    assert!(bytes.starts_with(b"CHRON1"));

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .env("CHRONICLE_KEY", format!("base64:{key_b64}"))
        .args(["recall", "--query", "secret", "--top", "1", "--no-touch"])
        .assert()
        .success()
        .stdout(contains("secret message"));

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["recall", "--query", "secret", "--top", "1", "--no-touch"])
        .assert()
        .failure()
        .stderr(contains("CHRONICLE_KEY"));
}

#[test]
fn forget_by_id_deletes_file() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["remember", "--id", "mem_x", "--msg", "to be deleted"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let path = tmp.path().join(".chronicle/short_term/mem_x.md");
    assert!(path.exists());

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["forget", "--id", "mem_x"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    assert!(!path.exists());

    let commits = git_output(tmp.path(), &["rev-list", "--count", "HEAD"]);
    let n: u32 = commits.trim().parse().expect("commit count");
    assert!(n >= 2);
}

#[test]
fn log_shows_chronicle_commits() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["remember", "--msg", "log test [[Rust]]"])
        .assert()
        .success();

    // Drain WAL to ensure commit is visible to `chronicle log`.
    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["log", "--limit", "1"])
        .assert()
        .success()
        .stdout(contains("Memory update"));
}

#[test]
fn async_commit_via_wal() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["--commit", "async", "remember", "--msg", "async commit"])
        .assert()
        .success();

    // Ensure WAL is drained deterministically.
    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let commits = git_output(tmp.path(), &["rev-list", "--count", "HEAD"]);
    let n: u32 = commits.trim().parse().expect("commit count");
    assert!(n >= 1);
}

#[test]
fn recall_touch_updates_hit_count_and_outputs_json() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args([
            "remember",
            "--id",
            "mem_touch",
            "--msg",
            "touch me [[Rust]]",
        ])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let path = tmp.path().join(".chronicle/short_term/mem_touch.md");
    let raw1 = fs::read_to_string(&path).expect("read memory file");
    assert!(raw1.contains("hit_count: 1"));

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["recall", "--query", "touch", "--top", "1", "--json"])
        .assert()
        .success()
        .stdout(contains("\"results\""))
        .stdout(contains("mem_touch"));

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let raw2 = fs::read_to_string(&path).expect("read memory file after touch");
    assert!(raw2.contains("hit_count: 2"));

    let commits = git_output(tmp.path(), &["rev-list", "--count", "HEAD"]);
    let n: u32 = commits.trim().parse().expect("commit count");
    assert!(n >= 2);
}

#[test]
fn forget_by_threshold_archives_long_term() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args([
            "remember",
            "--layer",
            "long",
            "--id",
            "mem_old",
            "--msg",
            "old memory",
        ])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let from = tmp.path().join(".chronicle/long_term/mem_old.md");
    let to = tmp.path().join(".chronicle/archive/mem_old.md");
    assert!(from.exists());
    assert!(!to.exists());

    // Very high threshold => archive even fresh memories.
    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["forget", "--threshold", "0.99"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    assert!(!from.exists());
    assert!(to.exists());

    let commits = git_output(tmp.path(), &["rev-list", "--count", "HEAD"]);
    let n: u32 = commits.trim().parse().expect("commit count");
    assert!(n >= 2);
}

#[test]
fn branch_merge_fast_forward_and_delete() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["remember", "--msg", "seed"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let base = git_output(tmp.path(), &["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["branch", "create", "hypothesis-ff"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["remember", "--msg", "branch change"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["branch", "checkout", &base])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["branch", "merge", "hypothesis-ff"])
        .assert()
        .success();

    // After merge, it is fully merged and can be deleted.
    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["branch", "delete", "hypothesis-ff"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["branch", "list"])
        .assert()
        .success()
        .stdout(contains("hypothesis-ff").not());
}

#[test]
fn completions_and_status_smoke_and_commit_off() {
    let tmp = TempDir::new().expect("tempdir");

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["init", "--git-init"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["status"])
        .assert()
        .success()
        .stdout(contains("short_term: 0"));

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(contains("chronicle"));

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["remember", "--id", "mem_seed", "--msg", "seed"])
        .assert()
        .success();

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let commits_before = git_output(tmp.path(), &["rev-list", "--count", "HEAD"]);
    let before: u32 = commits_before.trim().parse().expect("commit count");
    assert!(before >= 1);

    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args([
            "--commit",
            "off",
            "remember",
            "--id",
            "mem_off",
            "--msg",
            "no commit",
        ])
        .assert()
        .success();

    // No WAL tasks should exist, but running is safe.
    Command::cargo_bin("chronicle")
        .expect("bin")
        .current_dir(tmp.path())
        .args(["wal", "run"])
        .assert()
        .success();

    let commits_after = git_output(tmp.path(), &["rev-list", "--count", "HEAD"]);
    let after: u32 = commits_after.trim().parse().expect("commit count");
    assert_eq!(after, before);

    assert!(tmp.path().join(".chronicle/short_term/mem_off.md").exists());
}
