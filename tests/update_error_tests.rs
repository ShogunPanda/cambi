mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, init_repo, seed_single_file_repo};

fn run_type_save_fail(repo: &std::path::Path, expected: &str) {
  let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cambi"));
  cmd.current_dir(repo).args(["update"]);
  cmd.assert().failure().stderr(predicate::str::contains(expected));
}

#[test]
fn fails_on_invalid_package_json() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "package.json", "{\"name\":\"x\",\"version\":\"1.2.3\"}\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  fs::write(repo.path().join("package.json"), "{not-json}\n").expect("break json");
  run_type_save_fail(repo.path(), "Invalid JSON");
}

#[test]
fn fails_on_missing_package_json_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "package.json", "{\"name\":\"x\"}\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  run_type_save_fail(repo.path(), "No 'version' field found");
}

#[test]
fn fails_on_invalid_pyproject_toml() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "pyproject.toml", "[project\nversion='1.2.3'\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  run_type_save_fail(repo.path(), "Invalid TOML");
}

#[test]
fn fails_on_pyproject_without_supported_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "pyproject.toml", "[project]\nname='x'\n\n[tool]\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  run_type_save_fail(repo.path(), "No supported version field found");
}

#[test]
fn fails_on_gemspec_without_version() {
  let repo = init_repo();
  seed_single_file_repo(
    &repo,
    "x.gemspec",
    "Gem::Specification.new do |spec|\n  spec.name = 'x'\nend\n",
  );
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  run_type_save_fail(repo.path(), "No spec.version assignment found");
}

#[test]
fn fails_on_mix_exs_without_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "mix.exs", "def project do\n  [name: :x]\nend\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  run_type_save_fail(repo.path(), "No version: field found");
}

#[test]
fn fails_on_pubspec_without_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "pubspec.yaml", "name: x\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  run_type_save_fail(repo.path(), "No 'version' field found");
}

#[test]
fn fails_on_package_swift_without_version() {
  let repo = init_repo();
  seed_single_file_repo(&repo, "Package.swift", "let name = \"x\"\n");
  fs::write(repo.path().join("a.txt"), "x").expect("write");
  commit_with_date(repo.path(), "feat: add", "2026-02-22T00:00:00Z");
  run_type_save_fail(repo.path(), "No supported version assignment found");
}
