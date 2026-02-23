mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, init_repo, seed_single_file_repo};

#[test]
fn release_dry_run_autodetects_repo_from_cargo() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"0.1.0\"\nrepository=\"https://github.com/octo/r\"\n",
  );
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  crate::common::git(repo.path(), &["tag", "v0.2.0"]);

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["release", "--dry-run"]);
  cmd.assert().success().stdout(predicate::str::contains("octo/r"));
}

#[test]
fn release_fails_when_owner_repo_cannot_be_detected() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "VERSION", "0.1.0\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  crate::common::git(repo.path(), &["tag", "v0.2.0"]);

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["release", "--dry-run"]);
  cmd
    .assert()
    .failure()
    .stderr(predicate::str::contains("Cannot determine GitHub owner/repo"));
}

#[test]
fn release_fails_without_token_when_not_dry_run() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"0.1.0\"\nrepository=\"https://github.com/octo/r\"\n",
  );
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  crate::common::git(repo.path(), &["tag", "v0.2.0"]);

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["release"]);
  cmd
    .assert()
    .failure()
    .stderr(predicate::str::contains("Missing GitHub token"));
}
