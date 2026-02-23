use cambi::conventional::{BumpLevel, infer_bump};

#[test]
fn detects_major_from_bang() {
  assert_eq!(infer_bump("feat(api)!: redesign", ""), BumpLevel::Major);
}

#[test]
fn detects_major_from_footer() {
  assert_eq!(
    infer_bump("feat(api): redesign", "something\nBREAKING CHANGE: api"),
    BumpLevel::Major
  );
}

#[test]
fn detects_minor_from_feat() {
  assert_eq!(infer_bump("feat(ui): add button", ""), BumpLevel::Minor);
}

#[test]
fn defaults_to_patch() {
  assert_eq!(infer_bump("docs: update", ""), BumpLevel::Patch);
  assert_eq!(infer_bump("non conventional message", ""), BumpLevel::Patch);
}

#[test]
fn supports_breaking_change_dash_footer_and_as_str() {
  assert_eq!(
    infer_bump("refactor(core): update", "BREAKING-CHANGE: api"),
    BumpLevel::Major
  );

  assert_eq!(BumpLevel::Patch.as_str(), "patch");
  assert_eq!(BumpLevel::Minor.as_str(), "minor");
  assert_eq!(BumpLevel::Major.as_str(), "major");
}
