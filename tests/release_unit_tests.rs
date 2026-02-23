use cambi::release::{
  normalize_release_version, parse_github_repo_from_url, release_tag, release_title, render_release_body,
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
