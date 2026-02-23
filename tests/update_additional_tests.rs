mod common;

use std::fs;

use assert_cmd::Command;

use crate::common::{commit_with_date, init_repo, seed_single_file_repo};

fn run_update(repo: &std::path::Path) -> String {
  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo).args(["update"]);
  let out = cmd.assert().success().get_output().stdout.clone();
  String::from_utf8(out).expect("utf8")
}

#[test]
fn cargo_toml_major_bump_is_saved() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );
  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "feat!: break all", "2026-02-22T00:00:00Z");

  let output = run_update(repo.path());
  assert_eq!(output, "Updated version to 2.0.0.\n");

  let cargo = fs::read_to_string(repo.path().join("Cargo.toml")).expect("read");
  assert!(
    cargo
      .lines()
      .any(|line| line.trim_start().starts_with("version") && line.contains("\"2.0.0\""))
  );
}

#[test]
fn cargo_toml_patch_bump_is_saved() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname=\"x\"\nversion=\"1.2.3\"\nrepository=\"https://github.com/octo/r\"\n",
  );
  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:00Z");

  let output = run_update(repo.path());
  assert_eq!(output, "Updated version to 1.2.4.\n");

  let cargo = fs::read_to_string(repo.path().join("Cargo.toml")).expect("read");
  assert!(
    cargo
      .lines()
      .any(|line| line.trim_start().starts_with("version") && line.contains("\"1.2.4\""))
  );
}

#[test]
fn cargo_toml_update_preserves_condensed_dependency_tables() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "Cargo.toml",
    "[package]\nname = \"x\"\nversion = \"1.2.3\"\nrepository = \"https://github.com/octo/r\"\n\n[dependencies]\nanyhow = { version = \"1.0.0\", features = [\"backtrace\"] }\n\n[dev-dependencies]\nassert_cmd = \"2.0.0\"\n",
  );
  fs::write(repo.path().join("src.rs"), "x").expect("write");
  commit_with_date(repo.path(), "fix: patch", "2026-02-22T00:00:00Z");

  run_update(repo.path());

  let cargo = fs::read_to_string(repo.path().join("Cargo.toml")).expect("read");
  assert!(cargo.contains("anyhow = { version = \"1.0.0\", features = [\"backtrace\"] }"));
  assert!(cargo.contains("[dev-dependencies]"));
  assert!(!cargo.contains("[dependencies.anyhow]"));
  assert!(!cargo.contains("[dev-dependencies.assert_cmd]"));
}
