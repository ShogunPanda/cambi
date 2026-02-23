use cambi::{
  changelog::{
    ChangelogSection, apply_default_sorting, extract_versions, format_date, render_section, with_prepended_section,
  },
  git::GitCommit,
};

#[test]
fn apply_default_sorting_covers_priorities_and_time_tiebreak() {
  let mut commits = vec![
    GitCommit {
      subject: "fix: z".to_string(),
      body: "".to_string(),
      time: 2,
    },
    GitCommit {
      subject: "feat: y".to_string(),
      body: "".to_string(),
      time: 1,
    },
    GitCommit {
      subject: "chore: BREAKING CHANGE api".to_string(),
      body: "".to_string(),
      time: 3,
    },
    GitCommit {
      subject: "refactor!: x".to_string(),
      body: "".to_string(),
      time: 3,
    },
    GitCommit {
      subject: "docs: a".to_string(),
      body: "".to_string(),
      time: 4,
    },
    GitCommit {
      subject: "fix: old".to_string(),
      body: "".to_string(),
      time: 1,
    },
  ];

  apply_default_sorting(&mut commits);

  assert!(commits[0].subject.contains("BREAKING CHANGE") || commits[0].subject.contains("!:"));
  assert!(commits[1].subject.contains("BREAKING CHANGE") || commits[1].subject.contains("!:"));
  assert_eq!(commits[2].subject, "feat: y");
  assert_eq!(commits[3].subject, "fix: z");
  assert_eq!(commits[4].subject, "fix: old");
}

#[test]
fn render_section_without_template_and_with_template() {
  let section = ChangelogSection {
    date: "2026-02-22".to_string(),
    version: "1.2.3".to_string(),
    commits: vec!["feat: add".to_string()],
  };

  let default_render = render_section(&section, None);
  assert!(default_render.contains("### 2026-02-22 / 1.2.3"));

  let custom = render_section(&section, Some("$DATE $VERSION\n$COMMITS"));
  assert_eq!(custom, "2026-02-22 1.2.3\n- feat: add");
}

#[test]
fn prepended_section_handles_empty_and_non_empty() {
  assert_eq!(with_prepended_section("", "A"), "A\n");
  assert_eq!(with_prepended_section("B", "A"), "A\n\nB\n");
}

#[test]
fn format_date_and_extract_versions_cover_non_matches() {
  assert_eq!(format_date(0), "1970-01-01");
  let versions = extract_versions("no section");
  assert!(versions.is_empty());
}
