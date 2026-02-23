use std::{collections::HashSet, fs, path::Path};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use git2::{Repository, Signature, StatusOptions};
use regex::Regex;
use semver::Version;

use crate::{
  cli::ChangelogArgs,
  config::EffectiveConfig,
  conventional::{BumpLevel, infer_bump},
  filters::CommitFilter,
  git::{GitCommit, GitTag, read_commits, read_commits_between_tags, read_tags},
};

pub struct ChangelogSection {
  pub date: String,
  pub version: String,
  pub commits: Vec<String>,
}

fn priority(subject: &str) -> i32 {
  if subject.contains("BREAKING CHANGE") || subject.contains("!:") {
    return 3;
  }

  if subject.starts_with("feat") {
    return 2;
  }

  if subject.starts_with("fix") {
    return 1;
  }

  0
}

pub fn normalize_tag_version(tag_name: &str) -> Option<Version> {
  let normalized = if tag_name.starts_with('v') {
    tag_name.trim_start_matches('v').to_string()
  } else {
    tag_name.to_string()
  };

  Version::parse(&normalized).ok()
}

pub fn bump_version(current: Option<Version>, bump: BumpLevel) -> Version {
  let mut next = current.unwrap_or_else(|| Version::new(0, 0, 0));

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

fn resolve_changelog_target(
  current: Option<Version>,
  target: Option<&str>,
  detected_bump: BumpLevel,
) -> Result<Version> {
  let Some(raw_target) = target else {
    return Ok(bump_version(current, detected_bump));
  };

  let normalized_target = raw_target.to_ascii_lowercase();

  match normalized_target.as_str() {
    "major" => Ok(bump_version(current, BumpLevel::Major)),
    "minor" => Ok(bump_version(current, BumpLevel::Minor)),
    "patch" => Ok(bump_version(current, BumpLevel::Patch)),
    _ => {
      let version = raw_target.trim_start_matches('v');
      Version::parse(version).map_err(|_| anyhow!("Invalid changelog target version '{raw_target}'"))
    }
  }
}

fn commit_changelog(commit_message: &str, verbose: bool) -> Result<()> {
  let repo = Repository::discover(".").context("Failed to discover git repository")?;
  let mut options = StatusOptions::new();
  options.include_untracked(false).recurse_untracked_dirs(false);

  let statuses = repo.statuses(Some(&mut options)).context("Failed to read git status")?;

  let changed_paths = statuses
    .iter()
    .filter_map(|entry| entry.path().map(|path| path.to_string()))
    .collect::<Vec<_>>();

  if changed_paths == vec!["CHANGELOG.md".to_string()] {
    let mut index = repo.index().context("Cannot open git index")?;
    index
      .add_path(Path::new("CHANGELOG.md"))
      .context("Cannot stage CHANGELOG.md")?;
    index.write().context("Cannot write git index")?;

    let tree_id = index.write_tree().context("Cannot write git tree")?;
    let tree = repo.find_tree(tree_id).context("Cannot find git tree")?;

    let signature = repo
      .signature()
      .or_else(|_| Signature::now("cambi", "cambi@localhost"))
      .context("Cannot build git signature")?;

    let mut parents = Vec::new();
    if let Some(oid) = repo.head().ok().and_then(|head| head.target()) {
      parents.push(repo.find_commit(oid).context("Cannot find HEAD commit")?);
    }

    let parent_refs = parents.iter().collect::<Vec<_>>();
    repo
      .commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &parent_refs,
      )
      .context("Cannot create git commit")?;
  } else if verbose {
    eprintln!("Skipping auto-commit: files changed are {:?}", changed_paths);
  }

  Ok(())
}

pub fn apply_default_sorting(commits: &mut [GitCommit]) {
  commits.sort_by(|a, b| {
    let by_priority = priority(&b.subject).cmp(&priority(&a.subject));
    if by_priority == std::cmp::Ordering::Equal {
      b.time.cmp(&a.time)
    } else {
      by_priority
    }
  });
}

pub fn extract_versions(markdown: &str) -> HashSet<String> {
  let re =
    Regex::new(r"(?m)^###\s+\d{4}-\d{2}-\d{2}\s*/\s*([0-9]+\.[0-9]+\.[0-9]+)\s*$").expect("version regex must compile");

  re.captures_iter(markdown)
    .filter_map(|capture| capture.get(1).map(|m| m.as_str().to_string()))
    .collect::<HashSet<_>>()
}

pub fn format_date(timestamp: i64) -> String {
  DateTime::<Utc>::from_timestamp(timestamp, 0)
    .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
    .format("%Y-%m-%d")
    .to_string()
}

pub fn render_section(section: &ChangelogSection, template: Option<&str>) -> String {
  if let Some(template) = template {
    let commits = section
      .commits
      .iter()
      .map(|entry| format!("- {entry}"))
      .collect::<Vec<_>>()
      .join("\n");

    return template
      .replace("$DATE", &section.date)
      .replace("$VERSION", &section.version)
      .replace("$COMMITS", &commits)
      .trim()
      .to_string();
  }

  let mut output = format!("### {} / {}\n\n", section.date, section.version);
  for commit in &section.commits {
    output.push_str("- ");
    output.push_str(commit);
    output.push('\n');
  }

  output.trim().to_string()
}

pub fn with_prepended_section(existing: &str, section_markdown: &str) -> String {
  let existing = existing.trim();
  if existing.is_empty() {
    return format!("{section_markdown}\n");
  }

  format!("{section_markdown}\n\n{existing}\n")
}

pub fn collect_releasable_commits(mut commits: Vec<GitCommit>, filter: &CommitFilter) -> Vec<GitCommit> {
  commits.retain(|commit| !filter.is_ignored(&commit.subject) && !commit.subject.starts_with("chore"));
  commits
}

fn render_tag_history_sections(tags: &[GitTag], filter: &CommitFilter, template: Option<&str>) -> Result<Vec<String>> {
  let mut historical = Vec::new();
  let mut previous_tag_name: Option<String> = None;

  for tag in tags.iter().rev() {
    let commits = collect_releasable_commits(
      read_commits_between_tags(previous_tag_name.as_deref(), &tag.name)?,
      filter,
    );

    if !commits.is_empty() {
      let mut commits = commits;
      apply_default_sorting(&mut commits);

      if let Some(version) = normalize_tag_version(&tag.name) {
        let date = format_date(commits.first().map(|commit| commit.time).unwrap_or(tag.time));
        let section = ChangelogSection {
          date,
          version: version.to_string(),
          commits: commits.into_iter().map(|commit| commit.subject).collect::<Vec<_>>(),
        };

        historical.push(render_section(&section, template));
      }
    }

    previous_tag_name = Some(tag.name.clone());
  }

  Ok(historical)
}

fn build_rebuild_output(config: &EffectiveConfig, filter: &CommitFilter, template: Option<&str>) -> Result<String> {
  let tags = read_tags(&config.tag_pattern)?;
  let historical = render_tag_history_sections(&tags, filter, template)?;
  let latest_version = tags.first().and_then(|tag| normalize_tag_version(&tag.name));

  let mut pending_commits = collect_releasable_commits(read_commits(None, &config.tag_pattern)?, filter);
  let mut sections = Vec::new();

  if !pending_commits.is_empty() {
    apply_default_sorting(&mut pending_commits);

    let bump = pending_commits
      .iter()
      .map(|commit| infer_bump(&commit.subject, &commit.body))
      .max()
      .unwrap_or(BumpLevel::Patch);

    let section = ChangelogSection {
      date: format_date(pending_commits.first().map(|commit| commit.time).unwrap_or(0)),
      version: bump_version(latest_version, bump).to_string(),
      commits: pending_commits
        .into_iter()
        .map(|commit| commit.subject)
        .collect::<Vec<_>>(),
    };

    sections.push(render_section(&section, template));
  }

  for section in historical.into_iter().rev() {
    sections.push(section);
  }

  if sections.is_empty() {
    return Ok(String::new());
  }

  Ok(format!("{}\n", sections.join("\n\n")))
}

pub fn execute_changelog_command(changelog_args: &ChangelogArgs, config: &EffectiveConfig) -> Result<()> {
  if changelog_args.rebuild && changelog_args.target.is_some() {
    return Err(anyhow!("Cannot combine --rebuild with an explicit changelog target"));
  }

  let changelog_path = Path::new("CHANGELOG.md");
  let existing = fs::read_to_string(changelog_path).unwrap_or_default();
  let existing_versions = extract_versions(&existing);
  let filter = CommitFilter::new(&config.ignore_patterns)?;
  let template = config.changelog_template.as_deref();

  if changelog_args.rebuild {
    let output = build_rebuild_output(config, &filter, template)?;

    if changelog_args.dry_run {
      println!("{output}");
      return Ok(());
    }

    fs::write(changelog_path, output).context("Failed to write CHANGELOG.md")?;

    if changelog_args.commit {
      let commit_message = changelog_args
        .commit_message
        .as_deref()
        .unwrap_or("chore: Updated CHANGELOG.md.");
      commit_changelog(commit_message, config.verbose)?;
    }

    return Ok(());
  }

  let tags = read_tags(&config.tag_pattern)?;
  let latest_version = tags.first().and_then(|tag| normalize_tag_version(&tag.name));

  let mut commits = collect_releasable_commits(read_commits(None, &config.tag_pattern)?, &filter);

  if commits.is_empty() {
    if config.verbose {
      eprintln!("No releasable commits found. CHANGELOG.md not updated.");
    }
    return Ok(());
  }

  apply_default_sorting(&mut commits);

  let bump = commits
    .iter()
    .map(|commit| infer_bump(&commit.subject, &commit.body))
    .max()
    .unwrap_or(BumpLevel::Patch);

  let next_version = resolve_changelog_target(latest_version, changelog_args.target.as_deref(), bump)?;
  let next_version_string = next_version.to_string();

  if existing_versions.contains(&next_version_string) {
    if config.verbose {
      eprintln!("Version {} already exists in CHANGELOG.md", next_version_string);
    }
    return Ok(());
  }

  let section = ChangelogSection {
    date: format_date(commits.first().map(|commit| commit.time).unwrap_or(0)),
    version: next_version_string.clone(),
    commits: commits.into_iter().map(|commit| commit.subject).collect::<Vec<_>>(),
  };

  let section_markdown = render_section(&section, template);
  let output = with_prepended_section(&existing, &section_markdown);

  if changelog_args.dry_run {
    println!("{output}");
    return Ok(());
  }

  fs::write(changelog_path, output).context("Failed to write CHANGELOG.md")?;

  if changelog_args.commit {
    let commit_message = changelog_args
      .commit_message
      .as_deref()
      .unwrap_or("chore: Updated CHANGELOG.md.");
    commit_changelog(commit_message, config.verbose)?;
  }

  Ok(())
}
