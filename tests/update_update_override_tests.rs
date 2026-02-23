mod common;

use std::fs;

use assert_cmd::Command;

use crate::common::{commit_with_date, init_repo, seed_single_file_repo};

#[test]
fn update_target_overrides_detected_bump() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );

  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["update", "major"]);
  cmd.assert().success().stdout("major\n");

  let cargo = fs::read_to_string(repo.path().join("Cargo.toml")).expect("read");
  assert!(cargo.contains("version = \"2.0.0\""));
}

#[test]
fn update_accepts_exact_version_and_skips_bump_detection_output() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );

  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["update", "3.4.5"]);
  cmd.assert().success().stdout("3.4.5\n");

  let cargo = fs::read_to_string(repo.path().join("Cargo.toml")).expect("read");
  assert!(cargo.contains("version = \"3.4.5\""));
}
