mod common;

use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;

use crate::common::{commit_with_date, create_repo, git, init_repo, seed_single_file_repo};

#[test]
fn commits_separate_update_and_script_changes_without_another_bump() {
  let repo = create_repo();
  fs::write(repo.path().join("generated.txt"), "old\n").expect("write generated file");
  fs::write(repo.path().join("removed.txt"), "old\n").expect("write removable file");
  commit_with_date(repo.path(), "feat: add generated files", "2026-02-22T00:00:00Z");

  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args(["update", "-l"])
    .assert()
    .success();

  let manifest = fs::read_to_string(repo.path().join("Cargo.toml")).expect("read manifest");
  let changelog = fs::read_to_string(repo.path().join("CHANGELOG.md")).expect("read changelog");
  let previous_head = git(repo.path(), &["rev-parse", "HEAD"]);
  // Simulate the script run after the version bump and before committing.
  fs::write(repo.path().join("generated.txt"), "0.2.0\n").expect("update generated file");
  fs::remove_file(repo.path().join("removed.txt")).expect("remove tracked file");
  fs::write(repo.path().join("untracked.txt"), "keep untracked\n").expect("write untracked file");
  fs::write(repo.path().join("staged.txt"), "already staged\n").expect("write staged file");
  git(repo.path(), &["add", "staged.txt"]);

  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args(["commit", "-t"])
    .assert()
    .success()
    .stdout("Committed version 0.2.0.\n");

  assert_eq!(
    git(repo.path(), &["log", "-1", "--pretty=%s"]).trim(),
    "chore: Updated version."
  );
  assert_eq!(git(repo.path(), &["rev-parse", "HEAD^"]), previous_head);
  assert_eq!(
    git(repo.path(), &["rev-parse", "v0.2.0"]),
    git(repo.path(), &["rev-parse", "HEAD"])
  );
  assert_eq!(git(repo.path(), &["show", "HEAD:generated.txt"]), "0.2.0\n");
  assert_eq!(git(repo.path(), &["show", "HEAD:Cargo.toml"]), manifest);
  assert_eq!(git(repo.path(), &["show", "HEAD:CHANGELOG.md"]), changelog);
  assert_eq!(git(repo.path(), &["status", "--short"]).trim(), "?? untracked.txt");
  let files = git(repo.path(), &["show", "--pretty=", "--name-only", "HEAD"]);
  assert!(files.lines().any(|line| line == "removed.txt"));
  assert!(files.lines().any(|line| line == "staged.txt"));
}

#[test]
fn commit_reads_all_supported_manifests_without_rewriting_them() {
  for (file, content) in [
    ("Cargo.toml", "[package]\nname='x'\nversion='1.2.3'\n"),
    ("package.json", "{\"name\":\"x\",\"version\":\"1.2.3\"}\n"),
    ("pyproject.toml", "[project]\nversion='1.2.3'\n"),
    ("pyproject.toml", "[tool.poetry]\nversion='1.2.3'\n"),
    (
      "x.gemspec",
      "Gem::Specification.new do |spec|\n  spec.version = '1.2.3'\nend\n",
    ),
    ("mix.exs", "def project do\n  [\n    version: \"1.2.3\",\n  ]\nend\n"),
    ("pubspec.yaml", "name: x\nversion: 1.2.3\n"),
    ("Package.swift", "let version = \"1.2.3\"\n"),
    ("Package.swift", "let p = Package(\n  version: \"1.2.3\",\n)\n"),
    ("version", "v1.2.3\n"),
    ("VERSION", "1.2.3\n"),
  ] {
    let repo = init_repo();
    seed_single_file_repo(&repo, file, content);
    fs::write(repo.path().join("notes.txt"), "staged notes\n").expect("write notes");
    git(repo.path(), &["add", "notes.txt"]);

    Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
      .current_dir(repo.path())
      .args(["commit", "-t", "-m", "chore: Prepared release."])
      .assert()
      .success();

    assert_eq!(
      fs::read_to_string(repo.path().join(file)).expect("read manifest"),
      content
    );
    assert_eq!(
      git(repo.path(), &["log", "-1", "--pretty=%s"]).trim(),
      "chore: Prepared release."
    );
    assert_eq!(
      git(repo.path(), &["rev-parse", "v1.2.3"]),
      git(repo.path(), &["rev-parse", "HEAD"])
    );
    assert!(!repo.path().join("CHANGELOG.md").exists());
  }
}

#[test]
fn dry_run_preserves_files_index_head_and_tags() {
  let repo = create_repo();
  fs::write(repo.path().join("src/lib.rs"), "pub fn updated() {}\n").expect("modify tracked file");
  fs::write(repo.path().join("CHANGELOG.md"), "Release notes\n").expect("write changelog");
  let head = git(repo.path(), &["rev-parse", "HEAD"]);
  let status = git(repo.path(), &["status", "--short"]);
  let index = fs::read(repo.path().join(".git/index")).expect("read index");
  let tags = git(repo.path(), &["tag", "--list"]);

  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args([
      "commit",
      "-d",
      "-t",
      "-m",
      "custom message",
      "-p",
      r"^release-\d+\.\d+\.\d+$",
    ])
    .assert()
    .success()
    .stdout(
      predicate::str::contains("with message 'custom message'")
        .and(predicate::str::contains("dry-run: would create tag release-0.1.0")),
    );

  assert_eq!(git(repo.path(), &["rev-parse", "HEAD"]), head);
  assert_eq!(git(repo.path(), &["status", "--short"]), status);
  assert_eq!(fs::read(repo.path().join(".git/index")).expect("read index"), index);
  assert_eq!(git(repo.path(), &["tag", "--list"]), tags);
  assert_eq!(
    fs::read_to_string(repo.path().join("CHANGELOG.md")).expect("read changelog"),
    "Release notes\n"
  );
}

#[test]
fn commit_uses_configured_tag_pattern() {
  let repo = create_repo();
  fs::write(
    repo.path().join("release.yml"),
    "tag_pattern: '^release-\\d+\\.\\d+\\.\\d+$'\n",
  )
  .expect("write config");
  fs::write(repo.path().join("src/lib.rs"), "pub fn updated() {}\n").expect("modify tracked file");

  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args(["commit", "-t", "-c", "release.yml"])
    .assert()
    .success();

  assert_eq!(
    git(repo.path(), &["rev-parse", "release-0.1.0"]),
    git(repo.path(), &["rev-parse", "HEAD"])
  );
}

#[test]
fn commit_without_tag_does_not_create_one() {
  let repo = create_repo();
  let tags = git(repo.path(), &["tag", "--list"]);
  fs::write(repo.path().join("src/lib.rs"), "pub fn updated() {}\n").expect("modify tracked file");

  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args(["commit"])
    .assert()
    .success();

  assert_eq!(git(repo.path(), &["tag", "--list"]), tags);
  assert_eq!(git(repo.path(), &["status", "--short"]), "");
}

#[test]
fn invalid_or_missing_manifest_fails_before_committing() {
  for content in [None, Some("{\"version\":\"invalid\"}\n")] {
    let repo = init_repo();
    seed_single_file_repo(&repo, "tracked.txt", "original\n");
    if let Some(content) = content {
      fs::write(repo.path().join("package.json"), content).expect("write invalid manifest");
    }
    fs::write(repo.path().join("tracked.txt"), "modified\n").expect("modify tracked file");
    let head = git(repo.path(), &["rev-parse", "HEAD"]);
    let status = git(repo.path(), &["status", "--short"]);

    Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
      .current_dir(repo.path())
      .args(["commit", "-t"])
      .assert()
      .failure()
      .stderr(predicate::str::contains(
        if content.is_some() {
          "Invalid semver"
        } else {
          "No supported package file found"
        },
      ));

    assert_eq!(git(repo.path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(git(repo.path(), &["status", "--short"]), status);
  }
}
