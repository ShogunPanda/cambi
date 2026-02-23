use cambi::{
  changelog::{bump_version, collect_releasable_commits, normalize_tag_version},
  conventional::BumpLevel,
  filters::CommitFilter,
  git::GitCommit,
};
use semver::Version;

#[test]
fn normalize_tag_version_handles_prefixed_and_invalid() {
  assert_eq!(normalize_tag_version("v1.2.3").expect("parse").to_string(), "1.2.3");
  assert_eq!(normalize_tag_version("1.2.3").expect("parse").to_string(), "1.2.3");
  assert!(normalize_tag_version("not-semver").is_none());
}

#[test]
fn bump_version_covers_all_levels() {
  assert_eq!(
    bump_version(Some(Version::new(1, 2, 3)), BumpLevel::Patch).to_string(),
    "1.2.4"
  );
  assert_eq!(
    bump_version(Some(Version::new(1, 2, 3)), BumpLevel::Minor).to_string(),
    "1.3.0"
  );
  assert_eq!(
    bump_version(Some(Version::new(1, 2, 3)), BumpLevel::Major).to_string(),
    "2.0.0"
  );
  assert_eq!(bump_version(None, BumpLevel::Patch).to_string(), "0.0.1");
}

#[test]
fn collect_releasable_commits_filters_chore_and_ignored() {
  let commits = vec![
    GitCommit {
      subject: "feat: add".to_string(),
      body: "".to_string(),
      time: 1,
    },
    GitCommit {
      subject: "chore: clean".to_string(),
      body: "".to_string(),
      time: 2,
    },
    GitCommit {
      subject: "wip: temp".to_string(),
      body: "".to_string(),
      time: 3,
    },
  ];

  let filter = CommitFilter::new(&["^wip: .+$".to_string()]).expect("regex");
  let kept = collect_releasable_commits(commits, &filter);
  assert_eq!(kept.len(), 1);
  assert_eq!(kept[0].subject, "feat: add");
}
