use std::{
  collections::HashMap,
  fs,
  path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

const DEFAULT_TAG_PATTERN: &str = r"^v\d+\.\d+\.\d+$";
const DEFAULT_IGNORE_PATTERNS: [&str; 7] = [
  r"^.+: fixup$",
  r"^.+: wip$",
  r"^fixup: .+$",
  r"^wip: .+$",
  r"^fixup$",
  r"^wip$",
  r"^Merge .+$",
];

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileConfig {
  pub token: Option<String>,
  pub owner: Option<String>,
  pub repo: Option<String>,
  pub tag_pattern: Option<String>,
  pub changelog_template: Option<String>,
  pub ignore_patterns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigOverrides {
  pub token: Option<String>,
  pub owner: Option<String>,
  pub repo: Option<String>,
  pub tag_pattern: Option<String>,
  pub verbose: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveConfig {
  pub token: Option<String>,
  pub owner: Option<String>,
  pub repo: Option<String>,
  pub tag_pattern: String,
  pub changelog_template: Option<String>,
  pub ignore_patterns: Vec<String>,
  pub verbose: bool,
}

impl EffectiveConfig {
  pub fn from_sources(config: Option<FileConfig>, env: &HashMap<String, String>, flags: ConfigOverrides) -> Self {
    let config = config.unwrap_or_default();

    let env_var = |key: &str| env.get(key).cloned();

    let token = flags
      .token
      .or_else(|| env_var("CAMBI_TOKEN"))
      .or_else(|| env_var("GH_RELEASE_TOKEN"))
      .or(config.token);

    let owner = flags.owner.or_else(|| env_var("CAMBI_OWNER")).or(config.owner);

    let repo = flags.repo.or_else(|| env_var("CAMBI_REPO")).or(config.repo);

    let tag_pattern = flags
      .tag_pattern
      .or_else(|| env_var("CAMBI_TAG_PATTERN"))
      .or(config.tag_pattern)
      .unwrap_or_else(|| DEFAULT_TAG_PATTERN.to_string());

    let changelog_template = env_var("CAMBI_CHANGELOG_TEMPLATE").or(config.changelog_template);

    let ignore_patterns = env_var("CAMBI_IGNORE_PATTERNS")
      .map(|raw| {
        raw
          .split(';')
          .map(str::trim)
          .filter(|entry| !entry.is_empty())
          .map(ToOwned::to_owned)
          .collect::<Vec<_>>()
      })
      .or(config.ignore_patterns)
      .unwrap_or_else(|| DEFAULT_IGNORE_PATTERNS.iter().map(ToString::to_string).collect());

    let verbose = flags
      .verbose
      .or_else(|| env_var("CAMBI_VERBOSE").map(|v| matches!(v.as_str(), "1" | "true" | "yes")))
      .unwrap_or(false);

    Self {
      token,
      owner,
      repo,
      tag_pattern,
      changelog_template,
      ignore_patterns,
      verbose,
    }
  }
}

fn read_config(path: &Path) -> Result<FileConfig> {
  let content = fs::read_to_string(path).with_context(|| format!("Cannot read config file: {}", path.display()))?;
  serde_yaml::from_str::<FileConfig>(&content)
    .with_context(|| format!("Invalid YAML in config file: {}", path.display()))
}

pub fn load_file(config_path_override: Option<&Path>) -> Result<Option<FileConfig>> {
  if let Some(path) = config_path_override {
    return read_config(path).map(Some);
  }

  let global = if let Some(home) = std::env::var_os("HOME") {
    PathBuf::from(home).join(".config/cambi.yml")
  } else {
    PathBuf::from(".config/cambi.yml")
  };

  let local = PathBuf::from("cambi.yml");

  let mut result = if global.exists() {
    Some(read_config(&global)?)
  } else {
    None
  };

  if local.exists() {
    let overlay = read_config(&local)?;
    let mut merged = result.unwrap_or_default();

    merged.token = overlay.token.or(merged.token);
    merged.owner = overlay.owner.or(merged.owner);
    merged.repo = overlay.repo.or(merged.repo);
    merged.tag_pattern = overlay.tag_pattern.or(merged.tag_pattern);
    merged.changelog_template = overlay.changelog_template.or(merged.changelog_template);
    merged.ignore_patterns = overlay.ignore_patterns.or(merged.ignore_patterns);

    result = Some(merged);
  }

  Ok(result)
}
