#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BumpLevel {
  Patch,
  Minor,
  Major,
}

impl BumpLevel {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Patch => "patch",
      Self::Minor => "minor",
      Self::Major => "major",
    }
  }
}

pub fn infer_bump(subject: &str, body: &str) -> BumpLevel {
  let header = subject.split_once(": ").map(|(prefix, _)| prefix).unwrap_or("");
  let header_breaking = header.ends_with('!');

  let header_without_breaking = if header_breaking {
    &header[..header.len().saturating_sub(1)]
  } else {
    header
  };

  let commit_type = header_without_breaking
    .split_once('(')
    .map(|(kind, _)| kind)
    .unwrap_or(header_without_breaking);

  let footer_breaking = body.lines().any(|line| {
    let normalized = line.trim_start();
    normalized.starts_with("BREAKING CHANGE:") || normalized.starts_with("BREAKING-CHANGE:")
  });

  if header_breaking || footer_breaking {
    return BumpLevel::Major;
  }

  if commit_type == "feat" {
    return BumpLevel::Minor;
  }

  BumpLevel::Patch
}
