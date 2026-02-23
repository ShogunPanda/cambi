use std::{
  fs,
  path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use regex::Regex;
use semver::Version;

use crate::{
  cli::{SemverArgs, UpdateArgs, VersionArgs},
  config::EffectiveConfig,
  conventional::{BumpLevel, infer_bump},
  filters::CommitFilter,
  git::{read_commits, read_tags},
};

fn bump_semver(current: Version, bump: BumpLevel) -> Version {
  let mut next = current;

  match bump {
    BumpLevel::Major => {
      next.major += 1;
      next.minor = 0;
      next.patch = 0;
    }
    BumpLevel::Minor => {
      next.minor += 1;
      next.patch = 0;
    }
    BumpLevel::Patch => {
      next.patch += 1;
    }
  }

  next
}

pub fn normalize_semver(raw: &str) -> Result<Version> {
  let normalized = raw.trim().trim_start_matches('v').to_string();
  Version::parse(&normalized).context(format!("Invalid semver '{}'", raw.trim()))
}

pub fn latest_tag_version(tag_pattern: &str) -> Result<Version> {
  let tags = read_tags(tag_pattern)?;

  for tag in tags {
    if let Ok(version) = normalize_semver(&tag.name) {
      return Ok(version);
    }
  }

  Ok(Version::new(0, 0, 0))
}

#[derive(Debug, Clone)]
pub enum UpdateTarget {
  Bump(BumpLevel),
  Exact(Version),
}

fn resolve_target_version(current: Version, target: &UpdateTarget) -> Version {
  match target {
    UpdateTarget::Bump(bump) => bump_semver(current, *bump),
    UpdateTarget::Exact(version) => version.clone(),
  }
}

fn parse_update_target(target: Option<&str>, commits_bump: BumpLevel) -> Result<UpdateTarget> {
  let Some(raw_target) = target else {
    return Ok(UpdateTarget::Bump(commits_bump));
  };

  match raw_target.to_ascii_lowercase().as_str() {
    "major" => Ok(UpdateTarget::Bump(BumpLevel::Major)),
    "minor" => Ok(UpdateTarget::Bump(BumpLevel::Minor)),
    "patch" => Ok(UpdateTarget::Bump(BumpLevel::Patch)),
    _ => Ok(UpdateTarget::Exact(normalize_semver(raw_target)?)),
  }
}

fn detect_bump(from_tag: Option<&str>, config: &EffectiveConfig) -> Result<BumpLevel> {
  let commits = read_commits(from_tag, &config.tag_pattern)?;
  let filter = CommitFilter::new(&config.ignore_patterns)?;

  let bump = commits
    .into_iter()
    .filter(|commit| !filter.is_ignored(&commit.subject))
    .map(|commit| infer_bump(&commit.subject, &commit.body))
    .max()
    .unwrap_or(BumpLevel::Patch);

  Ok(bump)
}

pub fn update_cargo_toml_version(path: &Path, target: &UpdateTarget) -> Result<String> {
  let content = fs::read_to_string(path).context(format!("Cannot read {}", path.display()))?;
  let mut doc = content
    .parse::<toml_edit::DocumentMut>()
    .context(format!("Invalid TOML in {}", path.display()))?;

  let package = doc
    .get("package")
    .and_then(toml_edit::Item::as_table_like)
    .ok_or(anyhow!("No [package] section found in {}", path.display()))?;

  let current = package
    .get("version")
    .and_then(toml_edit::Item::as_str)
    .ok_or(anyhow!("No package version found in {}", path.display()))?;

  let next = resolve_target_version(normalize_semver(current)?, target);
  let next_string = next.to_string();
  doc["package"]["version"] = toml_edit::value(next_string.clone());

  fs::write(path, doc.to_string()).context(format!("Cannot write {}", path.display()))?;

  Ok(next_string)
}

pub fn update_package_json_version(path: &Path, target: &UpdateTarget) -> Result<String> {
  let content = fs::read_to_string(path).context(format!("Cannot read {}", path.display()))?;
  let mut json: serde_json::Value =
    serde_json::from_str(&content).context(format!("Invalid JSON in {}", path.display()))?;

  let current = json
    .get("version")
    .and_then(|value| value.as_str())
    .ok_or(anyhow!("No 'version' field found in {}", path.display()))?;

  let next = resolve_target_version(normalize_semver(current)?, target);

  let object = json
    .as_object_mut()
    .ok_or(anyhow!("{} must contain a top-level object", path.display()))?;

  object.insert("version".to_string(), serde_json::Value::String(next.to_string()));

  fs::write(
    path,
    serde_json::to_string_pretty(&json).context("Cannot serialize package.json")? + "\n",
  )
  .context(format!("Cannot write {}", path.display()))?;

  Ok(next.to_string())
}

pub fn update_pyproject_toml_version(path: &Path, target: &UpdateTarget) -> Result<String> {
  let content = fs::read_to_string(path).context(format!("Cannot read {}", path.display()))?;
  let mut parsed: toml::Value = toml::from_str(&content).context(format!("Invalid TOML in {}", path.display()))?;

  if let Some(project_table) = parsed.get_mut("project").and_then(toml::Value::as_table_mut)
    && let Some(version) = project_table.get("version").and_then(toml::Value::as_str)
  {
    let next = resolve_target_version(normalize_semver(version)?, target);
    project_table.insert("version".to_string(), toml::Value::String(next.to_string()));

    fs::write(
      path,
      toml::to_string_pretty(&parsed).context("Cannot serialize pyproject.toml")? + "\n",
    )
    .context(format!("Cannot write {}", path.display()))?;

    return Ok(next.to_string());
  }

  if let Some(tool_table) = parsed.get_mut("tool").and_then(toml::Value::as_table_mut)
    && let Some(poetry_table) = tool_table.get_mut("poetry").and_then(toml::Value::as_table_mut)
    && let Some(version) = poetry_table.get("version").and_then(toml::Value::as_str)
  {
    let next = resolve_target_version(normalize_semver(version)?, target);
    poetry_table.insert("version".to_string(), toml::Value::String(next.to_string()));

    fs::write(
      path,
      toml::to_string_pretty(&parsed).context("Cannot serialize pyproject.toml")? + "\n",
    )
    .context(format!("Cannot write {}", path.display()))?;

    return Ok(next.to_string());
  }

  Err(anyhow!(
    "No supported version field found in {} (expected [project].version or [tool.poetry].version)",
    path.display()
  ))
}

pub fn find_gemspec_path() -> Result<PathBuf> {
  let entries = fs::read_dir(".").context("Cannot read current directory")?;

  for entry in entries {
    let path = entry.context("Cannot read directory entry")?.path();
    if path.extension().and_then(|ext| ext.to_str()) == Some("gemspec") {
      return Ok(path);
    }
  }

  Err(anyhow!("No .gemspec file found in current directory"))
}

pub fn update_gemspec_version(path: &Path, target: &UpdateTarget) -> Result<String> {
  let content = fs::read_to_string(path).context(format!("Cannot read {}", path.display()))?;
  let re = Regex::new(r#"^(?P<indent>\s*spec\.version\s*=\s*["'])(?P<version>[^"']+)(?P<suffix>["']\s*)$"#)
    .expect("gemspec version regex must compile");

  let mut lines = Vec::new();
  let mut updated: Option<String> = None;

  for line in content.lines() {
    if updated.is_none()
      && let Some(captures) = re.captures(line)
    {
      let current = captures
        .name("version")
        .map(|m| m.as_str())
        .ok_or(anyhow!("Cannot parse spec.version in {}", path.display()))?;
      let next = resolve_target_version(normalize_semver(current)?, target);
      let prefix = captures.name("indent").map(|m| m.as_str()).unwrap_or("");
      let suffix = captures.name("suffix").map(|m| m.as_str()).unwrap_or("");

      lines.push(format!("{prefix}{next}{suffix}"));
      updated = Some(next.to_string());
      continue;
    }

    lines.push(line.to_string());
  }

  let updated = updated.ok_or(anyhow!("No spec.version assignment found in {}", path.display()))?;
  fs::write(path, format!("{}\n", lines.join("\n"))).context(format!("Cannot write {}", path.display()))?;

  Ok(updated)
}

pub fn update_plain_version_file(path: &Path, target: &UpdateTarget, tag_pattern: &str) -> Result<String> {
  let current = if path.exists() {
    normalize_semver(&fs::read_to_string(path).context(format!("Cannot read {}", path.display()))?)?
  } else {
    latest_tag_version(tag_pattern)?
  };

  let next = resolve_target_version(current, target);
  fs::write(path, format!("{}\n", next)).context(format!("Cannot write {}", path.display()))?;

  Ok(next.to_string())
}

pub fn update_mix_exs_version(path: &Path, target: &UpdateTarget) -> Result<String> {
  let content = fs::read_to_string(path).context(format!("Cannot read {}", path.display()))?;
  let version_line = Regex::new(r#"^(?P<prefix>\s*version:\s*["'])(?P<version>[^"']+)(?P<suffix>["']\s*,?\s*)$"#)
    .expect("mix.exs version regex must compile");

  let mut lines = Vec::new();
  let mut updated: Option<String> = None;

  for line in content.lines() {
    if updated.is_none()
      && let Some(captures) = version_line.captures(line)
    {
      let current = captures
        .name("version")
        .map(|m| m.as_str())
        .ok_or(anyhow!("Cannot parse version in {}", path.display()))?;
      let next = resolve_target_version(normalize_semver(current)?, target);
      let prefix = captures.name("prefix").map(|m| m.as_str()).unwrap_or("");
      let suffix = captures.name("suffix").map(|m| m.as_str()).unwrap_or("");

      lines.push(format!("{prefix}{next}{suffix}"));
      updated = Some(next.to_string());
      continue;
    }

    lines.push(line.to_string());
  }

  let updated = updated.ok_or(anyhow!("No version: field found in {}", path.display()))?;
  fs::write(path, format!("{}\n", lines.join("\n"))).context(format!("Cannot write {}", path.display()))?;

  Ok(updated)
}

pub fn update_pubspec_yaml_version(path: &Path, target: &UpdateTarget) -> Result<String> {
  let content = fs::read_to_string(path).context(format!("Cannot read {}", path.display()))?;
  let mut parsed: serde_yaml::Value =
    serde_yaml::from_str(&content).context(format!("Invalid YAML in {}", path.display()))?;

  let current = parsed
    .get("version")
    .and_then(serde_yaml::Value::as_str)
    .ok_or(anyhow!("No 'version' field found in {}", path.display()))?;

  let next = resolve_target_version(normalize_semver(current)?, target);

  let map = parsed
    .as_mapping_mut()
    .ok_or(anyhow!("{} must contain a top-level mapping", path.display()))?;

  map.insert(
    serde_yaml::Value::String("version".to_string()),
    serde_yaml::Value::String(next.to_string()),
  );

  fs::write(
    path,
    serde_yaml::to_string(&parsed).context("Cannot serialize pubspec.yaml")?,
  )
  .context(format!("Cannot write {}", path.display()))?;

  Ok(next.to_string())
}

pub fn update_package_swift_version(path: &Path, target: &UpdateTarget) -> Result<String> {
  let content = fs::read_to_string(path).context(format!("Cannot read {}", path.display()))?;
  let variable_line =
    Regex::new(r#"^(?P<prefix>\s*(?:let|var)\s+version\s*=\s*["'])(?P<version>[^"']+)(?P<suffix>["']\s*)$"#)
      .expect("Package.swift variable regex must compile");
  let argument_line = Regex::new(r#"^(?P<prefix>\s*version\s*:\s*["'])(?P<version>[^"']+)(?P<suffix>["']\s*,?\s*)$"#)
    .expect("Package.swift argument regex must compile");

  let mut lines = Vec::new();
  let mut updated: Option<String> = None;

  for line in content.lines() {
    if updated.is_none()
      && let Some(captures) = variable_line.captures(line).or_else(|| argument_line.captures(line))
    {
      let current = captures
        .name("version")
        .map(|m| m.as_str())
        .ok_or(anyhow!("Cannot parse version in {}", path.display()))?;
      let next = resolve_target_version(normalize_semver(current)?, target);
      let prefix = captures.name("prefix").map(|m| m.as_str()).unwrap_or("");
      let suffix = captures.name("suffix").map(|m| m.as_str()).unwrap_or("");

      lines.push(format!("{prefix}{next}{suffix}"));
      updated = Some(next.to_string());
      continue;
    }

    lines.push(line.to_string());
  }

  let updated = updated.ok_or_else(|| {
    anyhow!(
      "No supported version assignment found in {} (expected let/var version = \"x.y.z\" or version: \"x.y.z\")",
      path.display()
    )
  })?;

  fs::write(path, format!("{}\n", lines.join("\n"))).context(format!("Cannot write {}", path.display()))?;

  Ok(updated)
}

fn apply_update_target(target: &UpdateTarget, config: &EffectiveConfig) -> Result<String> {
  let cargo_toml = Path::new("Cargo.toml");
  if cargo_toml.exists() {
    return update_cargo_toml_version(cargo_toml, target);
  }

  let package_json = Path::new("package.json");
  if package_json.exists() {
    return update_package_json_version(package_json, target);
  }

  let pyproject_toml = Path::new("pyproject.toml");
  if pyproject_toml.exists() {
    return update_pyproject_toml_version(pyproject_toml, target);
  }

  if let Ok(gemspec_path) = find_gemspec_path() {
    return update_gemspec_version(&gemspec_path, target);
  }

  let mix_exs = Path::new("mix.exs");
  if mix_exs.exists() {
    return update_mix_exs_version(mix_exs, target);
  }

  let pubspec_yaml = Path::new("pubspec.yaml");
  if pubspec_yaml.exists() {
    return update_pubspec_yaml_version(pubspec_yaml, target);
  }

  let package_swift = Path::new("Package.swift");
  if package_swift.exists() {
    return update_package_swift_version(package_swift, target);
  }

  let version_lower = Path::new("version");
  if version_lower.exists() {
    return update_plain_version_file(version_lower, target, &config.tag_pattern);
  }

  let version_upper = Path::new("VERSION");
  if version_upper.exists() {
    return update_plain_version_file(version_upper, target, &config.tag_pattern);
  }

  Err(anyhow!(
    "No supported package file found (Cargo.toml, package.json, pyproject.toml, *.gemspec, mix.exs, pubspec.yaml, \
     Package.swift, or version/VERSION)"
  ))
}

pub fn execute_version(version_args: &VersionArgs, config: &EffectiveConfig) -> Result<()> {
  let current = if let Some(from_tag) = version_args.from_tag.as_deref() {
    normalize_semver(from_tag)?
  } else {
    latest_tag_version(&config.tag_pattern)?
  };

  println!("{}", current);
  Ok(())
}

pub fn execute_semver(semver_args: &SemverArgs, config: &EffectiveConfig) -> Result<()> {
  let bump = detect_bump(semver_args.from_tag.as_deref(), config)?;
  println!("{}", bump.as_str());

  Ok(())
}

pub fn execute_update(update_args: &UpdateArgs, config: &EffectiveConfig) -> Result<()> {
  let detected_bump = detect_bump(update_args.from_tag.as_deref(), config)?;
  let target = parse_update_target(update_args.target.as_deref(), detected_bump)?;

  let updated = apply_update_target(&target, config)?;
  println!("Updated version to {}.", updated);

  Ok(())
}
