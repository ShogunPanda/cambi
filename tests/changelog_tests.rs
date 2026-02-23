use cambi::changelog::extract_versions;

#[test]
fn extracts_versions_from_default_headers() {
  let markdown = "### 2026-02-22 / 1.2.3\n\n- feat: x\n\n### 2026-02-21 / 1.2.2\n\n- fix: y\n";
  let versions = extract_versions(markdown);
  assert!(versions.contains("1.2.3"));
  assert!(versions.contains("1.2.2"));
}
