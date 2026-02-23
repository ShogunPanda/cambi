mod common;

use std::fs;

use cambi::git::{read_commits, read_commits_between_tags, read_tags};
use serial_test::serial;

use crate::common::{commit_with_date, create_repo, git};

#[test]
#[serial]
fn reads_tags_with_pattern() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "v0.2.0"]);
  git(repo.path(), &["tag", "other"]);

  let old = std::env::current_dir().expect("cwd");
  std::env::set_current_dir(repo.path()).expect("set cwd");
  let tags = read_tags(r"^v\d+\.\d+\.\d+$").expect("read tags");
  std::env::set_current_dir(old).expect("restore cwd");

  assert_eq!(tags.len(), 2);
  assert_eq!(tags[0].name, "v0.2.0");
}

#[test]
#[serial]
fn reads_commits_since_latest_tag() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  let old = std::env::current_dir().expect("cwd");
  std::env::set_current_dir(repo.path()).expect("set cwd");
  let commits = read_commits(None, r"^v\d+\.\d+\.\d+$").expect("read commits");
  std::env::set_current_dir(old).expect("restore cwd");

  assert_eq!(commits.len(), 1);
  assert_eq!(commits[0].subject, "feat: add output");
}

#[test]
#[serial]
fn reads_commits_between_tags() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "v0.2.0"]);

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"y\"); }\n").expect("write");
  commit_with_date(repo.path(), "fix: tweak", "2026-02-23T10:00:00Z");
  git(repo.path(), &["tag", "v0.2.1"]);

  let old = std::env::current_dir().expect("cwd");
  std::env::set_current_dir(repo.path()).expect("set cwd");
  let commits = read_commits_between_tags(Some("v0.2.0"), "v0.2.1").expect("between tags");
  std::env::set_current_dir(old).expect("restore cwd");

  assert_eq!(commits.len(), 1);
  assert_eq!(commits[0].subject, "fix: tweak");
}

#[test]
#[serial]
fn invalid_tag_pattern_returns_error() {
  let repo = create_repo();
  let old = std::env::current_dir().expect("cwd");
  std::env::set_current_dir(repo.path()).expect("set cwd");
  let result = read_tags("(");
  std::env::set_current_dir(old).expect("restore cwd");

  assert!(result.is_err());
}

#[test]
#[serial]
fn invalid_tag_lookup_returns_error() {
  let repo = create_repo();
  let old = std::env::current_dir().expect("cwd");
  std::env::set_current_dir(repo.path()).expect("set cwd");
  let result = read_commits(Some("v9.9.9"), r"^v\d+\.\d+\.\d+$");
  std::env::set_current_dir(old).expect("restore cwd");

  assert!(result.is_err());
}

#[test]
#[serial]
fn read_commits_from_specific_tag() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  let old = std::env::current_dir().expect("cwd");
  std::env::set_current_dir(repo.path()).expect("set cwd");
  let commits = read_commits(Some("v0.1.0"), r"^v\d+\.\d+\.\d+$").expect("read commits");
  std::env::set_current_dir(old).expect("restore cwd");

  assert_eq!(commits.len(), 1);
  assert_eq!(commits[0].subject, "feat: add output");
}

#[test]
#[serial]
fn invalid_end_tag_between_tags_returns_error() {
  let repo = create_repo();
  let old = std::env::current_dir().expect("cwd");
  std::env::set_current_dir(repo.path()).expect("set cwd");
  let result = read_commits_between_tags(Some("v0.1.0"), "v9.9.9");
  std::env::set_current_dir(old).expect("restore cwd");

  assert!(result.is_err());
}
