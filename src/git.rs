use anyhow::{Context, Result};
use git2::{ObjectType, Oid, Repository, Sort};
use regex::Regex;

pub struct GitTag {
  pub name: String,
  pub oid: Oid,
  pub time: i64,
}

pub struct GitCommit {
  pub subject: String,
  pub body: String,
  pub time: i64,
}

pub fn read_tags(tag_pattern: &str) -> Result<Vec<GitTag>> {
  let repo = Repository::discover(".").context("Failed to discover git repository")?;
  let tag_regex = Regex::new(tag_pattern).context(format!("Invalid tag regex pattern: {tag_pattern}"))?;

  let mut tags = repo
    .tag_names(None)
    .context("Cannot read git tag names")?
    .iter()
    .flatten()
    .filter(|name| tag_regex.is_match(name))
    .filter_map(|name| {
      let object = repo.revparse_single(&format!("refs/tags/{name}")).ok()?;
      let commit = if object.kind() == Some(ObjectType::Commit) {
        object.into_commit().ok()?
      } else {
        object.peel_to_commit().ok()?
      };

      Some(GitTag {
        name: name.to_string(),
        oid: commit.id(),
        time: commit.time().seconds(),
      })
    })
    .collect::<Vec<_>>();

  tags.sort_by_key(|b| std::cmp::Reverse(b.time));
  Ok(tags)
}

fn read_commits_between_oids(start_oid: Option<Oid>, end_oid: Option<Oid>) -> Result<Vec<GitCommit>> {
  let repo = Repository::discover(".").context("Failed to discover git repository")?;

  let end_oid = if let Some(end_oid) = end_oid {
    end_oid
  } else {
    repo
      .head()
      .context("Cannot read git HEAD")?
      .target()
      .context("HEAD is not pointing to a direct commit")?
  };

  let mut revwalk = repo.revwalk().context("Cannot create git revwalk")?;
  revwalk
    .set_sorting(Sort::TIME)
    .context("Cannot configure git revwalk sorting")?;
  revwalk.push(end_oid).context("Cannot push end ref into revwalk")?;

  if let Some(start_oid) = start_oid {
    revwalk
      .hide(start_oid)
      .context(format!("Cannot hide start commit {start_oid}"))?;
  }

  let mut commits = Vec::new();

  for oid_result in revwalk {
    let oid = oid_result.context("Failed iterating git history")?;
    let commit = repo.find_commit(oid).context(format!("Cannot find commit {oid}"))?;

    let message = commit.message().unwrap_or_default();
    let mut lines = message.lines();
    let subject = lines.next().unwrap_or_default().trim().to_string();
    let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();

    if !subject.is_empty() {
      commits.push(GitCommit {
        subject,
        body,
        time: commit.time().seconds(),
      });
    }
  }

  Ok(commits)
}

pub fn read_commits_between_tags(from_tag: Option<&str>, to_tag: &str) -> Result<Vec<GitCommit>> {
  let repo = Repository::discover(".").context("Failed to discover git repository")?;

  let start_oid = if let Some(tag) = from_tag {
    Some(
      repo
        .revparse_single(tag)
        .context(format!("Cannot resolve tag '{tag}'"))?
        .peel_to_commit()
        .context(format!("Tag '{tag}' does not resolve to a commit"))?
        .id(),
    )
  } else {
    None
  };

  let end_oid = repo
    .revparse_single(to_tag)
    .context(format!("Cannot resolve tag '{to_tag}'"))?
    .peel_to_commit()
    .context(format!("Tag '{to_tag}' does not resolve to a commit"))?
    .id();

  read_commits_between_oids(start_oid, Some(end_oid))
}

pub fn read_commits(from_tag: Option<&str>, tag_pattern: &str) -> Result<Vec<GitCommit>> {
  let start_oid = if let Some(tag) = from_tag {
    let repo = Repository::discover(".").context("Failed to discover git repository")?;
    Some(
      repo
        .revparse_single(tag)
        .context(format!("Cannot resolve tag '{tag}'"))?
        .peel_to_commit()
        .context(format!("Tag '{tag}' does not resolve to a commit"))?
        .id(),
    )
  } else {
    read_tags(tag_pattern)?.first().map(|tag| tag.oid)
  };

  read_commits_between_oids(start_oid, None)
}
