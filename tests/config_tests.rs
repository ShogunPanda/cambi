use std::{collections::HashMap, env, fs, path::Path};

use cambi::config::{ConfigOverrides, EffectiveConfig, FileConfig, load_file};
use serial_test::serial;
use tempfile::TempDir;

fn write_config(path: &Path, content: &str) {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).expect("create config dir");
  }
  fs::write(path, content).expect("write config file");
}

#[test]
#[serial]
fn precedence_is_flags_over_env_over_config_over_defaults() {
  let config = FileConfig {
    token: Some("config-token".into()),
    owner: Some("config-owner".into()),
    repo: Some("config-repo".into()),
    tag_pattern: Some("config-tag".into()),
    changelog_template: Some("config-template".into()),
    ignore_patterns: Some(vec!["config-ignore".into()]),
  };

  let env = HashMap::from([
    ("GH_RELEASE_TOKEN".to_string(), "env-token".to_string()),
    ("CAMBI_OWNER".to_string(), "env-owner".to_string()),
    ("CAMBI_REPO".to_string(), "env-repo".to_string()),
    ("CAMBI_TAG_PATTERN".to_string(), "env-tag".to_string()),
    ("CAMBI_CHANGELOG_TEMPLATE".to_string(), "env-template".to_string()),
    ("CAMBI_IGNORE_PATTERNS".to_string(), "env-a; env-b".to_string()),
    ("CAMBI_VERBOSE".to_string(), "yes".to_string()),
  ]);

  let flags = ConfigOverrides {
    token: Some("flag-token".into()),
    owner: Some("flag-owner".into()),
    repo: Some("flag-repo".into()),
    tag_pattern: Some("flag-tag".into()),
    verbose: Some(true),
  };

  let resolved = EffectiveConfig::from_sources(Some(config), &env, flags);

  assert_eq!(resolved.token.as_deref(), Some("flag-token"));
  assert_eq!(resolved.owner.as_deref(), Some("flag-owner"));
  assert_eq!(resolved.repo.as_deref(), Some("flag-repo"));
  assert_eq!(resolved.tag_pattern, "flag-tag");
  assert_eq!(resolved.changelog_template.as_deref(), Some("env-template"));
  assert_eq!(resolved.ignore_patterns, vec!["env-a", "env-b"]);
  assert!(resolved.verbose);
}

#[test]
#[serial]
fn defaults_are_applied_when_no_source_provides_values() {
  let resolved = EffectiveConfig::from_sources(None, &HashMap::new(), ConfigOverrides::default());

  assert_eq!(resolved.tag_pattern, r"^v\d+\.\d+\.\d+$");
  assert_eq!(resolved.ignore_patterns.len(), 7);
  assert!(!resolved.verbose);
}

#[test]
#[serial]
fn load_file_reads_explicit_override() {
  let temp = TempDir::new().expect("temp dir");
  let config_path = temp.path().join("my-config.yml");
  write_config(&config_path, "token: abc\nowner: org\nrepo: proj\n");

  let loaded = load_file(Some(&config_path))
    .expect("load config")
    .expect("config exists");
  assert_eq!(loaded.token.as_deref(), Some("abc"));
  assert_eq!(loaded.owner.as_deref(), Some("org"));
  assert_eq!(loaded.repo.as_deref(), Some("proj"));
}

#[test]
#[serial]
fn load_file_merges_global_then_local_with_local_priority() {
  let temp = TempDir::new().expect("temp dir");
  let home = temp.path().join("home");
  let cwd = temp.path().join("cwd");
  fs::create_dir_all(&cwd).expect("create cwd");

  let old_home = env::var_os("HOME");
  let old_cwd = env::current_dir().expect("current dir");

  // SAFETY: test is serialized and restores process state.
  unsafe { env::set_var("HOME", &home) };
  env::set_current_dir(&cwd).expect("set cwd");

  write_config(
    &home.join(".config/cambi.yml"),
    "token: global-token\nowner: global-owner\nrepo: global-repo\n",
  );
  write_config(
    Path::new("cambi.yml"),
    "owner: local-owner\ntag_pattern: local-tag\nignore_patterns:\n  - local-ignore\n",
  );

  let loaded = load_file(None).expect("load config").expect("config exists");

  assert_eq!(loaded.token.as_deref(), Some("global-token"));
  assert_eq!(loaded.owner.as_deref(), Some("local-owner"));
  assert_eq!(loaded.repo.as_deref(), Some("global-repo"));
  assert_eq!(loaded.tag_pattern.as_deref(), Some("local-tag"));
  assert_eq!(loaded.ignore_patterns.unwrap_or_default(), vec!["local-ignore"]);

  if let Some(old_home) = old_home {
    // SAFETY: restoring process env var in test teardown.
    unsafe { env::set_var("HOME", old_home) };
  } else {
    // SAFETY: restoring process env var in test teardown.
    unsafe { env::remove_var("HOME") };
  }
  env::set_current_dir(old_cwd).expect("restore cwd");
}

#[test]
#[serial]
fn load_file_returns_error_for_invalid_yaml() {
  let temp = TempDir::new().expect("temp dir");
  let bad = temp.path().join("bad.yml");
  write_config(&bad, "token: [\n");

  let error = load_file(Some(&bad)).expect_err("should fail");
  assert!(error.to_string().contains("Invalid YAML"));
}

#[test]
#[serial]
fn env_can_enable_verbose_without_flag() {
  let env = HashMap::from([("CAMBI_VERBOSE".to_string(), "true".to_string())]);
  let resolved = EffectiveConfig::from_sources(None, &env, ConfigOverrides::default());
  assert!(resolved.verbose);
}

#[test]
#[serial]
fn env_cambi_token_beats_gh_release_token() {
  let env = HashMap::from([
    ("CAMBI_TOKEN".to_string(), "cambi-token".to_string()),
    ("GH_RELEASE_TOKEN".to_string(), "gh-token".to_string()),
  ]);

  let resolved = EffectiveConfig::from_sources(None, &env, ConfigOverrides::default());
  assert_eq!(resolved.token.as_deref(), Some("cambi-token"));
}

#[test]
#[serial]
fn ignore_patterns_empty_entries_are_trimmed() {
  let env = HashMap::from([("CAMBI_IGNORE_PATTERNS".to_string(), "a;; b ; ".to_string())]);
  let resolved = EffectiveConfig::from_sources(None, &env, ConfigOverrides::default());
  assert_eq!(resolved.ignore_patterns, vec!["a", "b"]);
}

#[test]
#[serial]
fn verbose_flag_false_overrides_true_env() {
  let env = HashMap::from([("CAMBI_VERBOSE".to_string(), "true".to_string())]);
  let resolved = EffectiveConfig::from_sources(
    None,
    &env,
    ConfigOverrides {
      verbose: Some(false),
      ..ConfigOverrides::default()
    },
  );
  assert!(!resolved.verbose);
}

#[test]
#[serial]
fn load_file_none_when_no_configs_exist() {
  let temp = TempDir::new().expect("temp dir");
  let home = temp.path().join("home-none");
  let cwd = temp.path().join("cwd-none");
  fs::create_dir_all(&cwd).expect("create cwd");

  let old_home = env::var_os("HOME");
  let old_cwd = env::current_dir().expect("current dir");

  // SAFETY: test is serialized and restores process state.
  unsafe { env::set_var("HOME", &home) };
  env::set_current_dir(&cwd).expect("set cwd");

  let loaded = load_file(None).expect("load config");
  assert!(loaded.is_none());

  if let Some(old_home) = old_home {
    // SAFETY: restoring process env var in test teardown.
    unsafe { env::set_var("HOME", old_home) };
  } else {
    // SAFETY: restoring process env var in test teardown.
    unsafe { env::remove_var("HOME") };
  }
  env::set_current_dir(old_cwd).expect("restore cwd");
}

#[test]
#[serial]
fn load_file_works_when_home_is_unset() {
  let temp = TempDir::new().expect("temp dir");
  let cwd = temp.path().join("cwd-no-home");
  fs::create_dir_all(&cwd).expect("create cwd");

  let old_home = env::var_os("HOME");
  let old_cwd = env::current_dir().expect("current dir");

  // SAFETY: test is serialized and restores process state.
  unsafe { env::remove_var("HOME") };
  env::set_current_dir(&cwd).expect("set cwd");

  let loaded = load_file(None).expect("load config");
  assert!(loaded.is_none());

  if let Some(old_home) = old_home {
    // SAFETY: restoring process env var in test teardown.
    unsafe { env::set_var("HOME", old_home) };
  }
  env::set_current_dir(old_cwd).expect("restore cwd");
}
