use std::{env, fs, path::Path};

use cambi::{
  conventional::BumpLevel,
  version::{
    UpdateTarget, find_gemspec_path, latest_tag_version, normalize_semver, update_cargo_toml_version,
    update_gemspec_version, update_mix_exs_version, update_package_json_version, update_package_swift_version,
    update_plain_version_file, update_pubspec_yaml_version, update_pyproject_toml_version,
  },
};
use serial_test::serial;
use tempfile::TempDir;

#[test]
fn normalize_semver_handles_v_prefix_and_invalid() {
  assert_eq!(normalize_semver("v1.2.3").expect("parse").to_string(), "1.2.3");
  assert!(normalize_semver("nope").is_err());
}

#[test]
fn update_cargo_toml_invalid_toml_errors() {
  let temp = TempDir::new().expect("tmp");
  let file = temp.path().join("Cargo.toml");
  fs::write(&file, "[package\n").expect("write");
  assert!(update_cargo_toml_version(&file, &UpdateTarget::Bump(BumpLevel::Patch)).is_err());
}

#[test]
fn update_package_json_top_level_not_object_errors() {
  let temp = TempDir::new().expect("tmp");
  let file = temp.path().join("package.json");
  fs::write(&file, "[]").expect("write");
  assert!(update_package_json_version(&file, &UpdateTarget::Bump(BumpLevel::Patch)).is_err());
}

#[test]
fn update_pyproject_supports_poetry_branch() {
  let temp = TempDir::new().expect("tmp");
  let file = temp.path().join("pyproject.toml");
  fs::write(&file, "[tool.poetry]\nversion='1.2.3'\n").expect("write");
  let new_v = update_pyproject_toml_version(&file, &UpdateTarget::Bump(BumpLevel::Patch)).expect("update");
  assert_eq!(new_v, "1.2.4");
}

#[test]
#[serial]
fn find_gemspec_path_errors_when_missing() {
  let temp = TempDir::new().expect("tmp");
  let old = env::current_dir().expect("cwd");
  env::set_current_dir(temp.path()).expect("set cwd");
  let result = find_gemspec_path();
  env::set_current_dir(old).expect("restore");
  assert!(result.is_err());
}

#[test]
fn update_gemspec_preserves_double_quotes() {
  let temp = TempDir::new().expect("tmp");
  let file = temp.path().join("x.gemspec");
  fs::write(&file, "spec.version = \"1.2.3\"\n").expect("write");
  let new_v = update_gemspec_version(&file, &UpdateTarget::Bump(BumpLevel::Minor)).expect("update");
  assert_eq!(new_v, "1.3.0");
}

#[test]
fn update_plain_version_file_uses_existing_file() {
  let temp = TempDir::new().expect("tmp");
  let file = temp.path().join("VERSION");
  fs::write(&file, "1.2.3\n").expect("write");
  let new_v =
    update_plain_version_file(&file, &UpdateTarget::Bump(BumpLevel::Patch), r"^v\d+\.\d+\.\d+$").expect("update");
  assert_eq!(new_v, "1.2.4");
}

#[test]
fn update_mix_exs_with_single_quotes_is_supported() {
  let temp = TempDir::new().expect("tmp");
  let file = temp.path().join("mix.exs");
  fs::write(&file, "  version: '1.2.3',\n").expect("write");
  let new_v = update_mix_exs_version(&file, &UpdateTarget::Bump(BumpLevel::Patch)).expect("update");
  assert_eq!(new_v, "1.2.4");
}

#[test]
fn update_pubspec_yaml_non_mapping_errors() {
  let temp = TempDir::new().expect("tmp");
  let file = temp.path().join("pubspec.yaml");
  fs::write(&file, "- 1\n- 2\n").expect("write");
  assert!(update_pubspec_yaml_version(&file, &UpdateTarget::Bump(BumpLevel::Patch)).is_err());
}

#[test]
#[serial]
fn latest_tag_version_returns_zero_when_no_semver_tags() {
  let temp = TempDir::new().expect("tmp");
  let old = env::current_dir().expect("cwd");
  env::set_current_dir(temp.path()).expect("set cwd");

  std::process::Command::new("git")
    .arg("init")
    .current_dir(temp.path())
    .output()
    .expect("git init");

  let v = latest_tag_version(r"^v\d+\.\d+\.\d+$").expect("version");
  let _ = env::set_current_dir(&old);
  if env::current_dir().is_err() {
    env::set_current_dir(env!("CARGO_MANIFEST_DIR")).expect("restore manifest dir");
  }
  assert_eq!(v.to_string(), "0.0.0");
}

#[test]
#[serial]
fn latest_tag_version_skips_non_semver_tag_names_when_pattern_matches_all() {
  let temp = TempDir::new().expect("tmp");
  let old = env::current_dir().expect("cwd");
  env::set_current_dir(temp.path()).expect("set cwd");

  std::process::Command::new("git")
    .args(["init", "-q"])
    .current_dir(temp.path())
    .output()
    .expect("git init");
  std::process::Command::new("git")
    .args(["config", "user.email", "tests@example.com"])
    .current_dir(temp.path())
    .output()
    .expect("git config email");
  std::process::Command::new("git")
    .args(["config", "user.name", "Tests"])
    .current_dir(temp.path())
    .output()
    .expect("git config name");

  fs::write(temp.path().join("README.md"), "x\n").expect("write");
  std::process::Command::new("git")
    .args(["add", "."])
    .current_dir(temp.path())
    .output()
    .expect("git add");
  std::process::Command::new("git")
    .args(["commit", "-m", "chore: init"])
    .current_dir(temp.path())
    .output()
    .expect("git commit");
  std::process::Command::new("git")
    .args(["tag", "not-a-semver"])
    .current_dir(temp.path())
    .output()
    .expect("git tag");

  let v = latest_tag_version(r".+").expect("version");
  let _ = env::set_current_dir(&old);
  if env::current_dir().is_err() {
    env::set_current_dir(env!("CARGO_MANIFEST_DIR")).expect("restore manifest dir");
  }
  assert_eq!(v.to_string(), "0.0.0");
}

#[test]
fn update_package_swift_var_form_supported() {
  let temp = TempDir::new().expect("tmp");
  let file = temp.path().join("Package.swift");
  fs::write(&file, "var version = \"1.2.3\"\n").expect("write");
  let new_v = update_package_swift_version(&file, &UpdateTarget::Bump(BumpLevel::Patch)).expect("update");
  assert_eq!(new_v, "1.2.4");
  assert!(Path::new(&file).exists());
}
