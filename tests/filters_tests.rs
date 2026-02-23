use cambi::filters::CommitFilter;

#[test]
fn matches_custom_patterns() {
  let filter = CommitFilter::new(&[r"^wip: .+$".to_string()]).expect("valid regex");
  assert!(filter.is_ignored("wip: test"));
}

#[test]
fn detects_merge_commits() {
  let filter = CommitFilter::new(&[]).expect("empty regex list is valid");
  assert!(filter.is_ignored("Merge branch 'feature'"));
}

#[test]
fn invalid_regex_returns_error() {
  let result = CommitFilter::new(&["(".to_string()]);
  assert!(result.is_err());
}
