mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, create_repo};

#[test]
fn changelog_target_overrides_detected_bump() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  commit_with_date(repo.path(), "fix: patch only", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["changelog", "major", "--dry-run"]);
  cmd.assert().success().stdout(predicate::str::contains("/ 1.0.0"));
}

#[test]
fn changelog_accepts_exact_version_target() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["changelog", "3.4.5", "--dry-run"]);
  cmd.assert().success().stdout(predicate::str::contains("/ 3.4.5"));
}
