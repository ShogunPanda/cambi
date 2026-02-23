mod common;

use std::fs;

use cambi::git::read_commits;

use crate::common::{commit_with_date, create_repo, git};

#[test]
fn annotated_tags_are_supported() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "-a", "v0.2.0", "-m", "release"]);

  let old = std::env::current_dir().expect("cwd");
  std::env::set_current_dir(repo.path()).expect("set cwd");
  let commits = read_commits(None, r"^v\d+\.\d+\.\d+$").expect("read commits");
  std::env::set_current_dir(old).expect("restore cwd");

  assert!(commits.is_empty());
}

#[test]
fn empty_subject_commits_are_skipped() {
  let repo = create_repo();

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  git(repo.path(), &["add", "."]);

  let output = std::process::Command::new("git")
    .current_dir(repo.path())
    .env("GIT_AUTHOR_DATE", "2026-02-22T10:00:00Z")
    .env("GIT_COMMITTER_DATE", "2026-02-22T10:00:00Z")
    .args(["commit", "--allow-empty-message", "-m", ""])
    .output()
    .expect("commit");
  assert!(output.status.success());

  let old = std::env::current_dir().expect("cwd");
  std::env::set_current_dir(repo.path()).expect("set cwd");
  let commits = read_commits(None, r"^v\d+\.\d+\.\d+$").expect("read commits");
  std::env::set_current_dir(old).expect("restore cwd");

  assert!(commits.is_empty());
}
