mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, init_repo, seed_single_file_repo};

#[test]
fn semver_outputs_text_format() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "VERSION", "1.2.3\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["semver"]);
  cmd.assert().success().stdout("patch\n");
}

#[test]
fn cargo_toml_missing_package_section_fails() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "Cargo.toml", "[workspace]\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["update"]);
  cmd
    .assert()
    .failure()
    .stderr(predicate::str::contains("No [package] section found"));
}

#[test]
fn cargo_toml_missing_version_fails() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "Cargo.toml", "[package]\nname=\"x\"\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["update"]);
  cmd
    .assert()
    .failure()
    .stderr(predicate::str::contains("No package version found"));
}

#[test]
fn update_verbose_prints_updated_version_message() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "VERSION", "1.2.3\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["--verbose", "update"]);
  cmd
    .assert()
    .success()
    .stdout(predicate::str::contains("Updated version to"));
}

#[test]
fn package_swift_version_argument_form_is_supported() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "Package.swift", "let p = Package(\n  version: \"1.2.3\",\n)\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["update"]);
  cmd.assert().success();

  let swift = fs::read_to_string(repo.path().join("Package.swift")).expect("read");
  assert!(swift.contains("version: \"1.3.0\""));
}

#[test]
fn version_prints_latest_tag_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "VERSION", "1.2.3\n");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["version"]);
  cmd.assert().success().stdout("0.1.0\n");
}

#[test]
fn version_with_from_tag_prints_normalized_input() {
  let repo = init_repo();

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["version", "--from-tag", "v1.2.3"]);
  cmd.assert().success().stdout("1.2.3\n");
}
