mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, create_repo};

#[test]
fn changelog_verbose_no_releasable_commits_prints_message() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write chore file");
  commit_with_date(repo.path(), "chore: internal cleanup", "2026-02-22T12:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["--verbose", "changelog"]);

  cmd
    .assert()
    .success()
    .stderr(predicate::str::contains("No releasable commits found"));
}

#[test]
fn changelog_uses_custom_template_from_config() {
  let repo = create_repo();
  fs::write(
    repo.path().join("cambi.yml"),
    "changelog_template: |\n  ### $DATE / $VERSION\n\n  $COMMITS\n",
  )
  .expect("write config");

  fs::write(repo.path().join("src/lib.rs"), "pub fn a() { println!(\"x\"); }\n").expect("write feat file");
  commit_with_date(repo.path(), "feat: add output", "2026-02-22T10:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["changelog", "--dry-run"]);

  cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("### 2026-02-22 / 0.2.0"))
    .stdout(predicate::str::contains("- feat: add output"));
}
