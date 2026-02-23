mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, create_repo, git};

#[test]
fn changelog_skips_when_next_version_already_exists() {
  let repo = create_repo();
  fs::write(repo.path().join("CHANGELOG.md"), "### 2026-02-22 / 0.2.0\n\n- old\n").expect("seed changelog");

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["--verbose", "changelog"]);

  cmd
    .assert()
    .success()
    .stderr(predicate::str::contains("already exists in CHANGELOG"));
}

#[test]
fn changelog_commit_verbose_skips_when_multiple_files_changed() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  // leave extra unstaged tracked change before changelog --commit
  fs::write(
    repo.path().join("src/lib.rs"),
    "pub fn a() { println!(\"changed\"); }\n",
  )
  .expect("modify extra file");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["--verbose", "changelog", "--commit"]);

  cmd
    .assert()
    .success()
    .stderr(predicate::str::contains("Skipping auto-commit: files changed are"));
}

#[test]
fn changelog_rebuild_without_sections_outputs_empty() {
  let repo = create_repo();
  // history has only chore init under v0.1.0

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["changelog", "--rebuild", "--dry-run"]);
  cmd.assert().success().stdout("\n");
}

#[test]
fn changelog_non_rebuild_uses_major_bump_for_breaking_commit() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  commit_with_date(repo.path(), "feat!: break api", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["changelog", "--dry-run"]);
  cmd.assert().success().stdout(predicate::str::contains("/ 1.0.0"));
}

#[test]
fn changelog_non_rebuild_uses_patch_bump_for_fix_commit() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  commit_with_date(repo.path(), "fix: patch only", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["changelog", "--dry-run"]);
  cmd.assert().success().stdout(predicate::str::contains("/ 0.1.1"));
}

#[test]
fn changelog_rebuild_ignores_non_semver_tags() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "not-a-version"]);
  git(repo.path(), &["tag", "v0.2.0"]);

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["changelog", "--rebuild", "--dry-run", "--tag-pattern", "^.*$"]);
  cmd.assert().success().stdout(predicate::str::contains("0.2.0"));
}

#[test]
fn changelog_rebuild_non_dry_run_with_commit_writes_and_commits() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["changelog", "--rebuild", "--commit"]);
  cmd.assert().success();

  let changelog = fs::read_to_string(repo.path().join("CHANGELOG.md")).expect("read changelog");
  assert!(changelog.contains("0.2.0"));
}
