mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, init_repo, seed_single_file_repo};

fn run_type_save(repo: &std::path::Path) {
  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo).args(["update"]);
  cmd.assert().success().stdout("minor\n");
}

#[test]
fn updates_package_json_version() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "package.json",
    "{\n  \"name\": \"x\",\n  \"version\": \"1.2.3\"\n}\n",
  );
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  run_type_save(repo.path());

  let pkg = fs::read_to_string(repo.path().join("package.json")).expect("read package");
  assert!(pkg.contains("\"version\": \"1.3.0\""));
}

#[test]
fn updates_pyproject_project_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "pyproject.toml", "[project]\nname='x'\nversion='1.2.3'\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  run_type_save(repo.path());

  let file = fs::read_to_string(repo.path().join("pyproject.toml")).expect("read");
  assert!(file.contains("version = \"1.3.0\""));
}

#[test]
fn updates_pyproject_poetry_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "pyproject.toml", "[tool.poetry]\nname='x'\nversion='1.2.3'\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  run_type_save(repo.path());

  let file = fs::read_to_string(repo.path().join("pyproject.toml")).expect("read");
  assert!(file.contains("version = \"1.3.0\""));
}

#[test]
fn updates_gemspec_version() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "x.gemspec",
    "Gem::Specification.new do |spec|\n  spec.version = '1.2.3'\nend\n",
  );
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  run_type_save(repo.path());

  let file = fs::read_to_string(repo.path().join("x.gemspec")).expect("read");
  assert!(file.contains("spec.version = '1.3.0'"));
}

#[test]
fn updates_mix_exs_version() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "mix.exs",
    "def project do\n  [\n    version: \"1.2.3\",\n  ]\nend\n",
  );
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  run_type_save(repo.path());

  let file = fs::read_to_string(repo.path().join("mix.exs")).expect("read");
  assert!(file.contains("version: \"1.3.0\""));
}

#[test]
fn updates_pubspec_yaml_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "pubspec.yaml", "name: x\nversion: 1.2.3\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  run_type_save(repo.path());

  let file = fs::read_to_string(repo.path().join("pubspec.yaml")).expect("read");
  assert!(file.contains("version: 1.3.0"));
}

#[test]
fn updates_package_swift_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "Package.swift", "let version = \"1.2.3\"\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  run_type_save(repo.path());

  let file = fs::read_to_string(repo.path().join("Package.swift")).expect("read");
  assert!(file.contains("let version = \"1.3.0\""));
}

#[test]
fn fails_when_no_supported_files() {
  let repo = init_repo();
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");

  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo.path()).args(["update"]);
  cmd
    .assert()
    .failure()
    .stderr(predicate::str::contains("No supported package file found"));
}

#[test]
fn updates_version_lowercase_file() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "version", "1.2.3\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  run_type_save(repo.path());

  let file = fs::read_to_string(repo.path().join("version")).expect("read");
  assert_eq!(file, "1.3.0\n");
}
