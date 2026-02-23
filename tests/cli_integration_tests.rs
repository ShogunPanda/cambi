mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, create_repo, git};

#[test]
fn semver_outputs_minor_for_feat_since_latest_tag() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).arg("semver");
  cmd.assert().success().stdout("minor\n");
}

#[test]
fn changelog_dry_run_generates_expected_changelog_section() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"y\"); }\n").expect("write fix file");
  commit_with_date(repo.path(), "fix: tweak output", "2026-02-22T11:00:00Z");

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"z\"); }\n").expect("write chore file");
  commit_with_date(repo.path(), "chore: internal cleanup", "2026-02-22T12:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["changelog", "--dry-run"]);

  let expected = "### 2026-02-22 / 0.2.0\n\n- feat: add output\n- fix: tweak output\n\n";

  cmd.assert().success().stdout(expected);
}

#[test]
fn release_notes_only_prints_generated_notes() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");
  git(repo.path(), &["tag", "v0.2.0"]);

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["release", "--notes-only"]);

  cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("- feat: add output"));
}

#[test]
fn release_notes_only_filters_non_releasable_commits() {
  let repo = create_repo();

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"feat\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"chore\"); }\n").expect("write chore file");
  commit_with_date(repo.path(), "chore: internal cleanup", "2026-02-22T11:00:00Z");

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"wip\"); }\n").expect("write wip file");
  commit_with_date(repo.path(), "wip: temporary commit", "2026-02-22T12:00:00Z");

  git(repo.path(), &["tag", "v0.2.0"]);

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["release", "--notes-only"]);

  cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("- feat: add output"))
    .stdout(predicate::str::contains("chore: internal cleanup").not())
    .stdout(predicate::str::contains("wip: temporary commit").not());
}

#[test]
fn release_notes_only_conflicts_with_rebuild() {
  let repo = create_repo();

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["release", "--notes-only", "--rebuild"]);

  cmd.assert().failure();
}

#[test]
fn release_fails_without_matching_tags() {
  let repo = create_repo();

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["release", "--notes-only", "--tag-pattern", "^does-not-match$"]);

  cmd
    .assert()
    .failure()
    .stderr(predicate::str::contains("No matching git tags found"));
}
