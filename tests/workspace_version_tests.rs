mod common;

use std::{fs, process::Command as ProcessCommand};

use assert_cmd::Command;
use cambi::{
  conventional::BumpLevel,
  version::{UpdateTarget, update_cargo_toml_version},
};
use common::{commit_with_date, git, init_repo};
use predicates::prelude::*;
use tempfile::TempDir;

fn workspace_repo(inherited_root: bool) -> TempDir {
  let repo = init_repo();
  let root_package = if inherited_root {
    "[package]\nname = 'root'\nversion.workspace = true\n"
  } else {
    ""
  };
  fs::write(
    repo.path().join("Cargo.toml"),
    format!(
      "{root_package}[workspace]\nmembers = ['member']\nresolver = '2'\n[workspace.package]\nversion = '1.2.3' # \
       shared version\n"
    ),
  )
  .unwrap();
  fs::create_dir_all(repo.path().join("member/src")).unwrap();
  fs::write(
    repo.path().join("member/Cargo.toml"),
    "[package]\nname = 'member'\nversion.workspace = true\n",
  )
  .unwrap();
  fs::write(repo.path().join("member/src/lib.rs"), "").unwrap();
  if inherited_root {
    fs::create_dir_all(repo.path().join("src")).unwrap();
    fs::write(repo.path().join("src/lib.rs"), "").unwrap();
  }
  let output = ProcessCommand::new("cargo")
    .args(["generate-lockfile", "--offline"])
    .current_dir(repo.path())
    .output()
    .unwrap();
  assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
  git(repo.path(), &["add", "-f", "Cargo.lock"]);
  commit_with_date(repo.path(), "chore: init", "2026-01-01T00:00:00Z");
  git(repo.path(), &["tag", "v1.2.3"]);
  repo
}

#[test]
fn workspace_updates_lockfile_before_commit_and_tag() {
  for inherited_root in [false, true] {
    let repo = workspace_repo(inherited_root);
    Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
      .current_dir(repo.path())
      .args(["update", "--commit", "--tag", "minor"])
      .assert()
      .success();

    let manifest = fs::read_to_string(repo.path().join("Cargo.toml")).unwrap();
    assert!(manifest.contains("1.3.0"));
    assert!(manifest.contains("# shared version"));
    if inherited_root {
      assert!(manifest.contains("version.workspace = true"));
    }
    assert!(
      fs::read_to_string(repo.path().join("member/Cargo.toml"))
        .unwrap()
        .contains("version.workspace = true")
    );
    let lock = git(repo.path(), &["show", "v1.3.0:Cargo.lock"]);
    assert!(lock.contains("version = \"1.3.0\""));
    assert!(!lock.contains("1.2.3"));
    assert!(git(repo.path(), &["status", "--short"]).trim().is_empty());
  }
}

#[test]
fn workspace_previews_do_not_write_files() {
  let repo = workspace_repo(false);
  let manifest = fs::read(repo.path().join("Cargo.toml")).unwrap();
  let lock = fs::read(repo.path().join("Cargo.lock")).unwrap();
  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args(["update", "--show", "2.0.0"])
    .assert()
    .success()
    .stdout("2.0.0\n");
  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args(["update", "--dry-run", "--commit", "--tag", "major"])
    .assert()
    .success()
    .stdout(predicate::str::contains("Cargo.toml, Cargo.lock"));
  assert_eq!(fs::read(repo.path().join("Cargo.toml")).unwrap(), manifest);
  assert_eq!(fs::read(repo.path().join("Cargo.lock")).unwrap(), lock);
  assert!(git(repo.path(), &["status", "--short"]).trim().is_empty());
}

#[test]
fn separate_commit_reads_workspace_version_and_includes_new_lockfile() {
  let repo = workspace_repo(false);
  git(repo.path(), &["rm", "Cargo.lock"]);
  commit_with_date(repo.path(), "chore: remove lockfile", "2026-01-02T00:00:00Z");
  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args(["update", "2.0.0"])
    .assert()
    .success();
  assert!(!repo.path().join("Cargo.lock").exists());
  let output = ProcessCommand::new("cargo")
    .args(["generate-lockfile", "--offline"])
    .current_dir(repo.path())
    .output()
    .unwrap();
  assert!(output.status.success());
  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args(["commit", "--tag"])
    .assert()
    .success();
  assert!(git(repo.path(), &["show", "v2.0.0:Cargo.lock"]).contains("2.0.0"));
  assert!(git(repo.path(), &["status", "--short"]).trim().is_empty());
}

#[test]
fn explicit_package_version_takes_precedence() {
  let repo = TempDir::new().unwrap();
  let path = repo.path().join("Cargo.toml");
  fs::write(
    &path,
    "[package]\nversion = '1.2.3'\n[workspace.package]\nversion = '9.0.0'\n",
  )
  .unwrap();
  assert_eq!(
    update_cargo_toml_version(&path, &UpdateTarget::Bump(BumpLevel::Patch)).unwrap(),
    "1.2.4"
  );
  let manifest = fs::read_to_string(path)
    .unwrap()
    .parse::<toml_edit::DocumentMut>()
    .unwrap();
  assert_eq!(manifest["package"]["version"].as_str(), Some("1.2.4"));
  assert_eq!(manifest["workspace"]["package"]["version"].as_str(), Some("9.0.0"));
}

#[test]
fn invalid_workspace_version_is_not_overwritten() {
  let repo = TempDir::new().unwrap();
  let path = repo.path().join("Cargo.toml");
  for content in [
    "[workspace.package]\n",
    "[workspace.package]\nversion = 123\n",
    "[workspace.package]\nversion = 'invalid'\n",
  ] {
    fs::write(&path, content).unwrap();
    assert!(update_cargo_toml_version(&path, &UpdateTarget::Bump(BumpLevel::Patch)).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), content);
  }
}

#[test]
fn cargo_failure_restores_manifest_and_does_not_tag() {
  let repo = workspace_repo(false);
  fs::remove_file(repo.path().join("member/src/lib.rs")).unwrap();
  let manifest = fs::read(repo.path().join("Cargo.toml")).unwrap();
  let lock = fs::read(repo.path().join("Cargo.lock")).unwrap();
  let head = git(repo.path(), &["rev-parse", "HEAD"]);
  Command::new(assert_cmd::cargo::cargo_bin!("cambi"))
    .current_dir(repo.path())
    .args(["update", "--commit", "--tag", "patch"])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Cannot update Cargo.lock"));
  assert_eq!(fs::read(repo.path().join("Cargo.toml")).unwrap(), manifest);
  assert_eq!(fs::read(repo.path().join("Cargo.lock")).unwrap(), lock);
  assert_eq!(git(repo.path(), &["rev-parse", "HEAD"]), head);
  assert_eq!(git(repo.path(), &["tag", "--list"]), "v1.2.3\n");
}
