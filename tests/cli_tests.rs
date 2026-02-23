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
fn changelog_commit_message_is_parsed() {
  let args = Args::parse_from([
    "cambi",
    "changelog",
    "--commit",
    "--commit-message",
    "custom message",
  ]);

  match args.command {
    Command::Changelog(changelog_args) => {
      assert!(changelog_args.commit);
      assert_eq!(changelog_args.commit_message.as_deref(), Some("custom message"));
    }
    _ => panic!("expected changelog command"),
  }
}

#[test]
fn changelog_commit_without_message_is_parsed() {
  let args = Args::parse_from(["cambi", "changelog", "--commit"]);

  match args.command {
    Command::Changelog(changelog_args) => {
      assert!(changelog_args.commit);
      assert_eq!(changelog_args.commit_message, None);
    }
    _ => panic!("expected changelog command"),
  }
}

#[test]
fn changelog_commit_message_requires_commit() {
  let parsed = Args::try_parse_from(["cambi", "changelog", "--commit-message", "custom"]);
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
fn update_commit_message_is_parsed() {
  let args = Args::parse_from([
    "cambi",
    "update",
    "major",
    "--commit",
    "--commit-message",
    "custom message",
  ]);

  match args.command {
    Command::Update(update_args) => {
      assert_eq!(update_args.target.as_deref(), Some("major"));
      assert!(update_args.commit);
      assert_eq!(update_args.commit_message.as_deref(), Some("custom message"));
    }
    _ => panic!("expected update command"),
  }
}

#[test]
fn update_commit_without_message_is_parsed() {
  let args = Args::parse_from(["cambi", "update", "major", "--commit"]);

  match args.command {
    Command::Update(update_args) => {
      assert_eq!(update_args.target.as_deref(), Some("major"));
      assert!(update_args.commit);
      assert_eq!(update_args.commit_message, None);
      assert!(!update_args.tag);
    }
    _ => panic!("expected update command"),
  }
}

#[test]
fn update_commit_message_requires_commit() {
  let parsed = Args::try_parse_from(["cambi", "update", "major", "--commit-message", "custom"]);
  assert!(parsed.is_err());
}

#[test]
fn update_tag_requires_commit() {
  let parsed = Args::try_parse_from(["cambi", "update", "--tag"]);
  assert!(parsed.is_err());
}

#[test]
fn update_tag_is_parsed_with_commit() {
  let args = Args::parse_from(["cambi", "update", "major", "--commit", "--tag"]);

  match args.command {
    Command::Update(update_args) => {
      assert_eq!(update_args.target.as_deref(), Some("major"));
      assert!(update_args.commit);
      assert!(update_args.tag);
    }
    _ => panic!("expected update command"),
  }
}

#[test]
fn update_short_flags_are_parsed() {
  let args = Args::parse_from(["cambi", "update", "major", "-o", "-m", "msg", "-t"]);

  match args.command {
    Command::Update(update_args) => {
      assert_eq!(update_args.target.as_deref(), Some("major"));
      assert!(update_args.commit);
      assert_eq!(update_args.commit_message.as_deref(), Some("msg"));
      assert!(update_args.tag);
    }
    _ => panic!("expected update command"),
  }
}

#[test]
fn changelog_short_flags_are_parsed() {
  let args = Args::parse_from(["cambi", "changelog", "-o", "-m", "msg"]);

  match args.command {
    Command::Changelog(changelog_args) => {
      assert!(changelog_args.commit);
      assert_eq!(changelog_args.commit_message.as_deref(), Some("msg"));
    }
    _ => panic!("expected changelog command"),
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
fn release_prerelease_short_flag_is_parsed() {
  let args = Args::parse_from(["cambi", "release", "minor", "-a"]);
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
