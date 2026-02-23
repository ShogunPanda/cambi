use cambi::cli::{Args, Command};
use clap::Parser;

#[test]
fn command_name_is_reported() {
  let args = Args::parse_from(["cambi", "version"]);
  assert_eq!(args.command.name(), "version");

  let args = Args::parse_from(["cambi", "semver"]);
  assert_eq!(args.command.name(), "semver");

  let args = Args::parse_from(["cambi", "update"]);
  assert_eq!(args.command.name(), "update");

  let args = Args::parse_from(["cambi", "changelog"]);
  assert_eq!(args.command.name(), "changelog");

  let args = Args::parse_from(["cambi", "release"]);
  assert_eq!(args.command.name(), "release");
}

#[test]
fn release_conflicts_are_enforced() {
  let parsed = Args::try_parse_from(["cambi", "release", "--notes-only", "--rebuild"]);
  assert!(parsed.is_err());

  let parsed = Args::try_parse_from(["cambi", "release", "1.2.3", "--rebuild"]);
  assert!(parsed.is_err());
}

#[test]
fn changelog_conflicts_are_enforced() {
  let parsed = Args::try_parse_from(["cambi", "changelog", "--commit", "--dry-run"]);
  assert!(parsed.is_err());
}

#[test]
fn version_from_tag_is_parsed() {
  let parsed = Args::try_parse_from(["cambi", "version", "--from-tag", "v1.2.3"]);
  assert!(parsed.is_ok());
}

#[test]
fn semver_rejects_current_flag() {
  let parsed = Args::try_parse_from(["cambi", "semver", "--current"]);
  assert!(parsed.is_err());
}

#[test]
fn update_target_is_parsed() {
  let args = Args::parse_from(["cambi", "update", "major"]);
  match args.command {
    Command::Update(update_args) => {
      assert_eq!(update_args.target.as_deref(), Some("major"));
    }
    _ => panic!("expected update command"),
  }
}

#[test]
fn release_target_is_parsed() {
  let args = Args::parse_from(["cambi", "release", "minor", "--prerelease"]);
  match args.command {
    Command::Release(release_args) => {
      assert_eq!(release_args.target.as_deref(), Some("minor"));
      assert!(release_args.prerelease);
    }
    _ => panic!("expected release command"),
  }
}

#[test]
fn single_letter_command_aliases_are_parsed() {
  let args = Args::parse_from(["cambi", "v"]);
  assert_eq!(args.command.name(), "version");

  let args = Args::parse_from(["cambi", "s"]);
  assert_eq!(args.command.name(), "semver");

  let args = Args::parse_from(["cambi", "u"]);
  assert_eq!(args.command.name(), "update");

  let args = Args::parse_from(["cambi", "c"]);
  assert_eq!(args.command.name(), "changelog");

  let args = Args::parse_from(["cambi", "r"]);
  assert_eq!(args.command.name(), "release");
}

#[test]
fn single_letter_option_aliases_are_parsed() {
  let args = Args::parse_from(["cambi", "-c", "cambi.yml", "-p", "^v", "u", "-f", "v1.2.3", "minor"]);

  assert_eq!(
    args.config.as_deref().map(|p| p.to_string_lossy().to_string()),
    Some("cambi.yml".to_string())
  );
  assert_eq!(args.tag_pattern.as_deref(), Some("^v"));

  match args.command {
    Command::Update(update_args) => {
      assert_eq!(update_args.from_tag.as_deref(), Some("v1.2.3"));
      assert_eq!(update_args.target.as_deref(), Some("minor"));
    }
    _ => panic!("expected update command"),
  }
}
