use std::{collections::HashSet, fs, path::Path};

use anyhow::{Context, Result, anyhow};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
  changelog::{apply_default_sorting, collect_releasable_commits},
  cli::ReleaseArgs,
  config::EffectiveConfig,
  filters::CommitFilter,
  git::{GitTag, read_commits_between_tags, read_tags},
};

#[derive(Debug, Clone)]
struct ReleaseCandidate {
  tag_name: String,
  title: String,
  body: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ExistingRelease {
  id: u64,
  tag_name: String,
  name: Option<String>,
  body: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ReleasePayload {
  tag_name: String,
  name: String,
  body: String,
  draft: bool,
  prerelease: bool,
}

pub fn parse_github_repo_from_url(url: &str) -> Option<(String, String)> {
  let re = Regex::new(r"github\.com[:/](?P<owner>[^/]+)/(?P<repo>[^/.]+)(?:\.git)?/?$")
    .expect("github repository regex must compile");
  let captures = re.captures(url)?;
  let owner = captures.name("owner")?.as_str().to_string();
  let repo = captures.name("repo")?.as_str().to_string();
  Some((owner, repo))
}

fn detect_owner_repo_from_files() -> Option<(String, String)> {
  if Path::new("Cargo.toml").exists() {
    let cargo = fs::read_to_string("Cargo.toml").ok()?;
    let re =
      Regex::new(r#"(?m)^\s*repository\s*=\s*"(?P<url>[^"]+)"\s*$"#).expect("cargo repository regex must compile");

    if let Some(captures) = re.captures(&cargo) {
      let url = captures.name("url")?.as_str();
      if let Some(parsed) = parse_github_repo_from_url(url) {
        return Some(parsed);
      }
    }
  }

  if Path::new("package.json").exists() {
    let package = fs::read_to_string("package.json").ok()?;
    let json: serde_json::Value = serde_json::from_str(&package).ok()?;

    if let Some(repository) = json.get("repository") {
      let mut repository_url = repository.as_str();
      if repository_url.is_none() {
        if let Some(value) = repository.get("url") {
          repository_url = value.as_str();
        }
      }

      if let Some(url) = repository_url {
        if let Some(parsed) = parse_github_repo_from_url(url) {
          return Some(parsed);
        }
      }
    }
  }

  None
}

pub fn normalize_release_version(version: &str) -> String { version.trim_start_matches('v').to_string() }

pub fn release_tag(tag: &str) -> String { format!("v{}", normalize_release_version(tag)) }

pub fn release_title(tag: &str) -> String { normalize_release_version(tag) }

pub fn render_release_body(commits: &[String]) -> String {
  if commits.is_empty() {
    return "- No notable changes.".to_string();
  }

  commits
    .iter()
    .map(|subject| format!("- {subject}"))
    .collect::<Vec<_>>()
    .join("\n")
}

fn build_release_candidates(tags: &[GitTag], filter: &CommitFilter) -> Result<Vec<ReleaseCandidate>> {
  let mut previous_tag_name: Option<String> = None;
  let mut candidates = Vec::new();

  for tag in tags.iter().rev() {
    let mut commits = collect_releasable_commits(
      read_commits_between_tags(previous_tag_name.as_deref(), &tag.name)?,
      filter,
    );
    apply_default_sorting(&mut commits);

    let subjects = commits.into_iter().map(|commit| commit.subject).collect::<Vec<_>>();

    candidates.push(ReleaseCandidate {
      tag_name: release_tag(&tag.name),
      title: release_title(&tag.name),
      body: render_release_body(&subjects),
    });

    previous_tag_name = Some(tag.name.clone());
  }

  Ok(candidates)
}

fn github_api_base() -> String {
  std::env::var("CAMBI_GITHUB_API_BASE").unwrap_or_else(|_| "https://api.github.com".to_string())
}

fn github_client() -> ureq::Agent { ureq::AgentBuilder::new().build() }

fn list_releases(owner: &str, repo: &str, token: &str) -> Result<Vec<ExistingRelease>> {
  let url = format!("{}/repos/{owner}/{repo}/releases?per_page=100", github_api_base());
  let response = github_client()
    .get(&url)
    .set("Accept", "application/vnd.github+json")
    .set("Authorization", &format!("Bearer {token}"))
    .set("X-GitHub-Api-Version", "2022-11-28")
    .set("User-Agent", "cambi")
    .call()
    .map_err(|error| anyhow!("GitHub API error while listing releases: {error}"))?;

  response
    .into_json::<Vec<ExistingRelease>>()
    .context("Failed to parse GitHub release list")
}

fn delete_release(owner: &str, repo: &str, token: &str, release_id: u64) -> Result<()> {
  let url = format!("{}/repos/{owner}/{repo}/releases/{release_id}", github_api_base());
  github_client()
    .delete(&url)
    .set("Accept", "application/vnd.github+json")
    .set("Authorization", &format!("Bearer {token}"))
    .set("X-GitHub-Api-Version", "2022-11-28")
    .set("User-Agent", "cambi")
    .call()
    .map_err(|error| anyhow!("GitHub API error while deleting release {release_id}: {error}"))?;

  Ok(())
}

fn create_release(owner: &str, repo: &str, token: &str, payload: &ReleasePayload) -> Result<()> {
  let url = format!("{}/repos/{owner}/{repo}/releases", github_api_base());
  github_client()
    .post(&url)
    .set("Accept", "application/vnd.github+json")
    .set("Authorization", &format!("Bearer {token}"))
    .set("X-GitHub-Api-Version", "2022-11-28")
    .set("User-Agent", "cambi")
    .send_json(serde_json::to_value(payload).context("Cannot serialize release payload")?)
    .map_err(|error| {
      anyhow!(
        "GitHub API error while creating release '{}': {error}",
        payload.tag_name
      )
    })?;

  Ok(())
}

fn update_release(owner: &str, repo: &str, token: &str, release_id: u64, payload: &ReleasePayload) -> Result<()> {
  let url = format!("{}/repos/{owner}/{repo}/releases/{release_id}", github_api_base());
  github_client()
    .patch(&url)
    .set("Accept", "application/vnd.github+json")
    .set("Authorization", &format!("Bearer {token}"))
    .set("X-GitHub-Api-Version", "2022-11-28")
    .set("User-Agent", "cambi")
    .send_json(serde_json::to_value(payload).context("Cannot serialize release payload")?)
    .map_err(|error| {
      anyhow!(
        "GitHub API error while updating release '{}': {error}",
        payload.tag_name
      )
    })?;

  Ok(())
}

fn resolve_owner_repo(config: &EffectiveConfig) -> Result<(String, String)> {
  if let (Some(owner), Some(repo)) = (config.owner.clone(), config.repo.clone()) {
    return Ok((owner, repo));
  }

  detect_owner_repo_from_files().ok_or(anyhow!(
    "Cannot determine GitHub owner/repo. Set CAMBI_OWNER and CAMBI_REPO, or use --owner/--repo."
  ))
}

fn resolve_token(config: &EffectiveConfig) -> Result<String> {
  config.token.clone().ok_or(anyhow!(
    "Missing GitHub token. Set GH_RELEASE_TOKEN/CAMBI_TOKEN or pass --token."
  ))
}

pub fn execute_release_command(args: &ReleaseArgs, config: &EffectiveConfig) -> Result<()> {
  let tags = read_tags(&config.tag_pattern)?;
  if tags.is_empty() {
    return Err(anyhow!(
      "No matching git tags found for pattern '{}'",
      config.tag_pattern
    ));
  }

  let filter = CommitFilter::new(&config.ignore_patterns)?;
  let candidates = build_release_candidates(&tags, &filter)?;
  let target_candidates = if args.rebuild {
    candidates
  } else {
    vec![
      candidates
        .last()
        .cloned()
        .ok_or(anyhow!("No release candidates produced from git tags"))?,
    ]
  };

  if args.notes_only {
    println!("{}", target_candidates[0].body);
    return Ok(());
  }

  let (owner, repo) = resolve_owner_repo(config)?;

  if args.dry_run {
    if args.rebuild {
      println!("dry-run: would rebuild GitHub releases for {owner}/{repo}");
    } else {
      println!("dry-run: would publish latest GitHub release for {owner}/{repo}");
    }

    for candidate in &target_candidates {
      println!(
        "dry-run: would upsert release tag={} title={}",
        candidate.tag_name, candidate.title
      );
    }

    if args.rebuild {
      println!("dry-run: rebuild would delete existing releases not matching git tags");
    }

    return Ok(());
  }

  let token = resolve_token(config)?;
  let mut existing = list_releases(&owner, &repo, &token)?;

  if args.rebuild {
    let target_tags = target_candidates
      .iter()
      .map(|candidate| candidate.tag_name.clone())
      .collect::<HashSet<_>>();

    for release in &existing {
      if !target_tags.contains(&release.tag_name) {
        delete_release(&owner, &repo, &token, release.id)?;
      }
    }

    existing = list_releases(&owner, &repo, &token)?;
  }

  for candidate in &target_candidates {
    let payload = ReleasePayload {
      tag_name: candidate.tag_name.clone(),
      name: candidate.title.clone(),
      body: candidate.body.clone(),
      draft: false,
      prerelease: false,
    };

    if let Some(found) = existing.iter().find(|release| release.tag_name == candidate.tag_name) {
      let same_name = found.name.as_deref() == Some(payload.name.as_str());
      let same_body = found.body.as_deref() == Some(payload.body.as_str());

      if same_name && same_body {
        continue;
      }

      update_release(&owner, &repo, &token, found.id, &payload)?;
    } else {
      create_release(&owner, &repo, &token, &payload)?;
    }
  }

  Ok(())
}
