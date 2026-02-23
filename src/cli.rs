use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(clap::Args, Debug)]
pub struct SemverArgs {
  /// Override start tag instead of auto-detecting latest version tag.
  #[arg(long, short = 'f')]
  pub from_tag: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct VersionArgs {
  /// Override start tag instead of auto-detecting latest version tag.
  #[arg(long, short = 'f')]
  pub from_tag: Option<String>,
}

#[derive(clap::Args, Debug, Default)]
pub struct UpdateArgs {
  /// Optional explicit update target (major|minor|patch or a semver like 1.2.3
  /// / v1.2.3).
  pub target: Option<String>,

  /// Override start tag instead of auto-detecting latest version tag.
  #[arg(long, short = 'f')]
  pub from_tag: Option<String>,

  /// Auto-commit updated version file.
  #[arg(long, short = 'o')]
  pub commit: bool,

  /// Custom commit message (requires --commit).
  #[arg(long, short = 'm', requires = "commit", value_name = "MESSAGE")]
  pub commit_message: Option<String>,

  /// Create a tag for the new version (requires --commit).
  #[arg(long, short = 't', requires = "commit")]
  pub tag: bool,
}

#[derive(clap::Args, Debug, Default)]
pub struct ChangelogArgs {
  /// Optional explicit changelog target (major|minor|patch or a semver like
  /// 1.2.3 / v1.2.3).
  #[arg(conflicts_with = "rebuild")]
  pub target: Option<String>,

  /// Regenerate CHANGELOG.md from the first commit.
  #[arg(long, short = 'r')]
  pub rebuild: bool,

  /// Auto-commit if CHANGELOG.md is the only changed file.
  #[arg(long, short = 'o', conflicts_with = "dry_run")]
  pub commit: bool,

  /// Custom commit message (requires --commit).
  #[arg(long, short = 'm', requires = "commit", value_name = "MESSAGE")]
  pub commit_message: Option<String>,

  /// Preview changes without writing files.
  #[arg(long, short = 'd', conflicts_with = "commit")]
  pub dry_run: bool,
}

#[derive(clap::Args, Debug, Default)]
pub struct ReleaseArgs {
  /// Optional explicit release target (major|minor|patch or a semver like
  /// 1.2.3 / v1.2.3).
  #[arg(conflicts_with = "rebuild")]
  pub target: Option<String>,

  /// Delete/recreate releases from scratch.
  #[arg(long, short = 'r', conflicts_with = "notes_only")]
  pub rebuild: bool,

  /// Print only the notes that would be used for the release body.
  #[arg(
    long,
    short = 'n',
    conflicts_with = "rebuild",
    conflicts_with = "token",
    conflicts_with = "owner",
    conflicts_with = "repo",
    conflicts_with = "dry_run"
  )]
  pub notes_only: bool,

  /// Override GitHub token.
  #[arg(long, short = 't', conflicts_with = "notes_only")]
  pub token: Option<String>,

  /// Override GitHub owner/organization.
  #[arg(long, short = 'o', conflicts_with = "notes_only")]
  pub owner: Option<String>,

  /// Override GitHub repository.
  #[arg(long, short = 'u', conflicts_with = "notes_only")]
  pub repo: Option<String>,

  /// Preview release actions without API calls.
  #[arg(long, short = 'd', conflicts_with = "notes_only")]
  pub dry_run: bool,

  /// Mark the GitHub release as a pre-release (requires positional target).
  #[arg(long, short = 'a', conflicts_with = "notes_only")]
  pub prerelease: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
  /// Print the current version.
  #[command(alias = "v")]
  Version(VersionArgs),
  /// Compute the next semantic bump type.
  #[command(alias = "s")]
  Semver(SemverArgs),
  /// Update project version files based on detected or explicit target.
  #[command(alias = "u")]
  Update(UpdateArgs),
  /// Update CHANGELOG.md with the next release section.
  #[command(alias = "c")]
  Changelog(ChangelogArgs),
  /// Publish releases on GitHub from git history derived by tags.
  #[command(alias = "r")]
  Release(ReleaseArgs),
}

impl Command {
  pub fn name(&self) -> &'static str {
    match self {
      Self::Version(_) => "version",
      Self::Semver(_) => "semver",
      Self::Update(_) => "update",
      Self::Changelog(_) => "changelog",
      Self::Release(_) => "release",
    }
  }
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
  /// Optional explicit config file path.
  #[arg(long, short = 'c', global = true)]
  pub config: Option<PathBuf>,

  /// Override the release tag matcher regex.
  #[arg(long, short = 'p', global = true)]
  pub tag_pattern: Option<String>,

  /// Enable verbose output.
  #[arg(long, short, global = true)]
  pub verbose: bool,

  #[command(subcommand)]
  pub command: Command,
}
