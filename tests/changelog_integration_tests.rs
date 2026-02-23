mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, create_repo, git};

#[test]
fn changelog_rebuild_dry_run_outputs_all_sections() {
  let repo = create_repo();

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "v0.2.0"]);

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"y\"); }\n").expect("write fix file");
  commit_with_date(repo.path(), "fix: tweak output", "2026-02-23T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["changelog", "--rebuild", "--dry-run"]);

  cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("### 2026-02-22 / 0.2.0"))
    .stdout(predicate::str::contains("### 2026-02-23 / 0.2.1"));
}

#[test]
fn changelog_with_commit_creates_git_commit() {
  let repo = create_repo();

  fs::write(
    repo.path().join("CHANGELOG.md"),
    "### 2026-01-01 / 0.1.0\n\n- chore: init\n",
  )
  .expect("seed changelog");
  crate::common::commit_with_date(repo.path(), "chore: seed changelog", "2026-01-02T00:00:00Z");

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["changelog", "--commit"]);
  cmd.assert().success();

  let mut log_cmd = Command::new("git");
  log_cmd.current_dir(repo.path()).args(["log", "-1", "--pretty=%s"]);
  log_cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("chore: Updated CHANGELOG.md."));
}

#[test]
fn changelog_with_custom_commit_message_uses_it() {
  let repo = create_repo();

  fs::write(
    repo.path().join("CHANGELOG.md"),
    "### 2026-01-01 / 0.1.0\n\n- chore: init\n",
  )
  .expect("seed changelog");
  crate::common::commit_with_date(repo.path(), "chore: seed changelog", "2026-01-02T00:00:00Z");

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args([
      "changelog",
      "--commit",
      "--commit-message",
      "chore: custom changelog commit",
    ]);
  cmd.assert().success();

  let mut log_cmd = Command::new("git");
  log_cmd.current_dir(repo.path()).args(["log", "-1", "--pretty=%s"]);
  log_cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("chore: custom changelog commit"));
}
