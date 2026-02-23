use std::collections::HashMap;

use anyhow::Result;
use cambi::{
  changelog::execute_changelog_command,
  cli::{Args, Command},
  config::{ConfigOverrides, EffectiveConfig, load_file},
  release::execute_release_command,
  version::{execute_semver, execute_update, execute_version},
};
use clap::Parser;

fn main() -> Result<()> {
  let args = match Args::try_parse() {
    Ok(opts) => opts,
    Err(e) => {
      match e.kind() {
        clap::error::ErrorKind::DisplayVersion => {
          println!("{}", env!("CARGO_PKG_VERSION"));
          return Ok(());
        }
        _ => {
          e.exit();
        }
      }
    }
  };

  let file_cfg = load_file(args.config.as_deref())?;

  let overrides = match &args.command {
    Command::Release(release) => {
      ConfigOverrides {
        token: release.token.clone(),
        owner: release.owner.clone(),
        repo: release.repo.clone(),
        tag_pattern: args.tag_pattern.clone(),
        verbose: Some(args.verbose),
      }
    }
    Command::Version(_) | Command::Semver(_) | Command::Update(_) | Command::Changelog(_) => {
      ConfigOverrides {
        tag_pattern: args.tag_pattern.clone(),
        verbose: Some(args.verbose),
        ..ConfigOverrides::default()
      }
    }
  };

  let config = EffectiveConfig::from_sources(file_cfg, &HashMap::from_iter(std::env::vars()), overrides);

  if config.verbose {
    eprintln!("Configuration loaded for command '{}'.", args.command.name());
  }

  match &args.command {
    Command::Version(version_args) => execute_version(version_args, &config)?,
    Command::Semver(semver_args) => execute_semver(semver_args, &config)?,
    Command::Update(update_args) => execute_update(update_args, &config)?,
    Command::Changelog(changelog_args) => execute_changelog_command(changelog_args, &config)?,
    Command::Release(release_args) => execute_release_command(release_args, &config)?,
  }

  Ok(())
}
