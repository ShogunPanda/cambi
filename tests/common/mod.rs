#![allow(dead_code)]

use std::{fs, path::Path, process::Command};

use tempfile::TempDir;

pub fn git(dir: &Path, args: &[&str]) -> String {
  let output = Command::new("git")
    .current_dir(dir)
    .args(args)
    .output()
    .expect("failed to run git");

  assert!(
    output.status.success(),
    "git {:?} failed: {}",
    args,
    String::from_utf8_lossy(&output.stderr)
  );

  String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn init_repo() -> TempDir {
  let temp = TempDir::new().expect("temp dir");
  git(temp.path(), &["init", "-q"]);
  git(temp.path(), &["config", "user.email", "tests@example.com"]);
  git(temp.path(), &["config", "user.name", "Tests"]);
  temp
}

pub fn create_repo() -> TempDir {
  let temp = init_repo();

  fs::write(
    temp.path().join("Cargo.toml"),
    "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nrepository = \"https://github.com/octo/repo\"\n",
  )
  .expect("write Cargo.toml");

  fs::create_dir_all(temp.path().join("src")).expect("create src dir");
  fs::write(temp.path().join("src/lib.rs"), "pub fn a() {}\n").expect("write src/lib.rs");

  commit_with_date(temp.path(), "chore: init", "2026-01-01T00:00:00Z");
  git(temp.path(), &["tag", "v0.1.0"]);

  temp
}

pub fn seed_single_file_repo(temp: &TempDir, file: &str, content: &str) {
  let path = temp.path().join(file);
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).expect("create parent dir");
  }
  fs::write(path, content).expect("write seed file");
  commit_with_date(temp.path(), "chore: init", "2026-01-01T00:00:00Z");
  git(temp.path(), &["tag", "v0.1.0"]);
}

pub fn commit_with_date(dir: &Path, message: &str, date: &str) {
  git(dir, &["add", "."]);

  let output = Command::new("git")
    .current_dir(dir)
    .env("GIT_AUTHOR_DATE", date)
    .env("GIT_COMMITTER_DATE", date)
    .args(["commit", "-m", message])
    .output()
    .expect("failed to run git commit");

  assert!(
    output.status.success(),
    "git commit failed: {}",
    String::from_utf8_lossy(&output.stderr)
  );
}
