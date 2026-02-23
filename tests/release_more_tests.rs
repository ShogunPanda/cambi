mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, init_repo, seed_single_file_repo};

#[test]
fn release_dry_run_autodetects_repo_from_package_json_string() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "package.json",
    "{\n  \"name\": \"x\",\n  \"repository\": \"https://github.com/octo/repo.git\"\n}\n",
  );
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  crate::common::git(repo.path(), &["tag", "v0.2.0"]);

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["release", "--dry-run"]);
  cmd.assert().success().stdout(predicate::str::contains("octo/repo"));
}

#[test]
fn release_dry_run_autodetects_repo_from_package_json_object() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "package.json",
    "{\n  \"name\": \"x\",\n  \"repository\": { \"url\": \"https://github.com/octo/repo.git\" }\n}\n",
  );
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  crate::common::git(repo.path(), &["tag", "v0.2.0"]);

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["release", "--dry-run"]);
  cmd.assert().success().stdout(predicate::str::contains("octo/repo"));
}

#[test]
fn release_notes_only_with_no_changes_reports_placeholder() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"0.1.0\"\nrepository=\"https://github.com/octo/r\"\n",
  );
  crate::common::git(repo.path(), &["tag", "v0.2.0"]);

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["release", "--notes-only"]);
  cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("No notable changes"));
}

#[test]
fn release_dry_run_with_invalid_package_repository_fails_detection() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "package.json",
    "{\n  \"name\": \"x\",\n  \"repository\": \"not-a-github-url\"\n}\n",
  );
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
fn release_rebuild_dry_run_mentions_delete_behavior() {
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
  cmd.current_dir(repo.path()).args(["release", "--rebuild", "--dry-run"]);
  cmd.assert().success().stdout(predicate::str::contains(
    "delete existing releases not matching git tags",
  ));
}

#[test]
fn release_target_overrides_created_release_version() {
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
  cmd.current_dir(repo.path()).args(["release", "0.9.0", "--dry-run"]);
  cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("tag=v0.9.0 title=0.9.0"));
}

#[test]
fn release_target_accepts_major_minor_patch_shorthands() {
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
  cmd.current_dir(repo.path()).args(["release", "minor", "--dry-run"]);
  cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("tag=v0.3.0 title=0.3.0"));
}
