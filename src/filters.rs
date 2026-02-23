use anyhow::Result;
use regex::Regex;

pub struct CommitFilter {
  patterns: Vec<Regex>,
}

impl CommitFilter {
  pub fn new(patterns: &[String]) -> Result<Self> {
    let patterns = patterns
      .iter()
      .map(|entry| Regex::new(entry))
      .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(Self { patterns })
  }

  pub fn is_ignored(&self, subject: &str) -> bool {
    subject.starts_with("Merge ") || self.patterns.iter().any(|pattern| pattern.is_match(subject))
  }
}
