mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, git, init_repo, seed_single_file_repo};

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

#[test]
fn update_with_commit_uses_default_message_even_if_repo_is_dirty() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );

  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:00Z");

  fs::write(repo.path().join("dirty.txt"), "dirty").expect("dirty change");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["update", "--commit"]);
  cmd.assert().success().stdout("Updated version to 1.2.4.\n");

  let subject = git(repo.path(), &["log", "-1", "--pretty=%s"]);
  assert_eq!(subject.trim(), "chore: Updated version.");

  let status = git(repo.path(), &["status", "--short"]);
  assert!(status.contains("?? dirty.txt"));
}

#[test]
fn update_with_commit_stages_all_tracked_changes_on_dirty_branch() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );

  fs::write(repo.path().join("tracked.txt"), "one\n").expect("write tracked file");
  commit_with_date(repo.path(), "chore: add tracked file", "2026-02-22T00:00:00Z");

  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:01Z");

  fs::write(repo.path().join("tracked.txt"), "two\n").expect("modify tracked file");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["update", "--commit"]);
  cmd.assert().success().stdout("Updated version to 1.2.4.\n");

  let tracked_content = git(repo.path(), &["show", "--pretty=", "--name-only", "HEAD"]);
  assert!(tracked_content.lines().any(|line| line == "Cargo.toml"));
  assert!(tracked_content.lines().any(|line| line == "tracked.txt"));

  let status = git(repo.path(), &["status", "--short"]);
  assert_eq!(status.trim(), "");
}

#[test]
fn update_with_commit_uses_custom_message() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );

  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["update", "--commit", "--commit-message", "chore: custom version commit"]);
  cmd.assert().success().stdout("Updated version to 1.2.4.\n");

  let subject = git(repo.path(), &["log", "-1", "--pretty=%s"]);
  assert_eq!(subject.trim(), "chore: custom version commit");
}

#[test]
fn update_with_commit_and_tag_creates_prefixed_tag_by_default() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );

  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["update", "--commit", "--tag"]);
  cmd.assert().success().stdout("Updated version to 1.2.4.\n");

  let tags = git(repo.path(), &["tag", "--list"]);
  assert!(tags.lines().any(|line| line == "v1.2.4"));
}

#[test]
fn update_with_commit_and_tag_uses_plain_version_when_pattern_matches_plain_semver() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );

  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd
    .current_dir(repo.path())
    .args(["--tag-pattern", "^\\d+\\.\\d+\\.\\d+$", "update", "--commit", "--tag"]);
  cmd.assert().success().stdout("Updated version to 1.2.4.\n");

  let tags = git(repo.path(), &["tag", "--list"]);
  assert!(tags.lines().any(|line| line == "1.2.4"));
}

#[test]
fn update_with_commit_and_tag_uses_prefix_from_tag_pattern() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );

  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args([
    "--tag-pattern",
    "^release-\\d+\\.\\d+\\.\\d+$",
    "update",
    "--commit",
    "--tag",
  ]);
  cmd.assert().success().stdout("Updated version to 1.2.4.\n");

  let tags = git(repo.path(), &["tag", "--list"]);
  assert!(tags.lines().any(|line| line == "release-1.2.4"));
}
