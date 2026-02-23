use cambi::{
  cli::ReleaseArgs,
  config::EffectiveConfig,
  release::{
    execute_release_command, normalize_release_version, parse_github_repo_from_url, release_tag, release_title,
    render_release_body,
  },
};

#[test]
fn parse_github_repo_from_url_handles_formats() {
  assert_eq!(
    parse_github_repo_from_url("git@github.com:org/repo.git"),
    Some(("org".to_string(), "repo".to_string()))
  );
  assert_eq!(
    parse_github_repo_from_url("https://github.com/org/repo"),
    Some(("org".to_string(), "repo".to_string()))
  );
  assert_eq!(parse_github_repo_from_url("https://example.com/org/repo"), None);
}

#[test]
fn release_helpers_cover_edge_cases() {
  assert_eq!(normalize_release_version("v1.2.3"), "1.2.3");
  assert_eq!(release_tag("v1.2.3"), "v1.2.3");
  assert_eq!(release_title("v1.2.3"), "1.2.3");
  assert_eq!(render_release_body(&[]), "- No notable changes.");
}

#[test]
fn release_rejects_target_with_rebuild_at_runtime() {
  let args = ReleaseArgs {
    target: Some("1.2.3".to_string()),
    rebuild: true,
    ..ReleaseArgs::default()
  };
  let config = EffectiveConfig {
    token: None,
    owner: None,
    repo: None,
    tag_pattern: r"^v\d+\.\d+\.\d+$".to_string(),
    changelog_template: None,
    ignore_patterns: vec![],
    verbose: false,
  };

  let error = execute_release_command(&args, &config).expect_err("must fail");
  assert!(
    error
      .to_string()
      .contains("Cannot combine --rebuild with an explicit release target")
  );
}

#[test]
fn release_rejects_prerelease_without_target_at_runtime() {
  let args = ReleaseArgs {
    prerelease: true,
    ..ReleaseArgs::default()
  };
  let config = EffectiveConfig {
    token: None,
    owner: None,
    repo: None,
    tag_pattern: r"^v\d+\.\d+\.\d+$".to_string(),
    changelog_template: None,
    ignore_patterns: vec![],
    verbose: false,
  };

  let error = execute_release_command(&args, &config).expect_err("must fail");
  assert!(
    error
      .to_string()
      .contains("--prerelease requires an explicit positional release target")
  );
}
