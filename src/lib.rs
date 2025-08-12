pub mod args;
pub mod utils;
pub mod notifiers;

use regex::Regex;
use anyhow::{Result, Context};

/// Matching engine: either literal substring or regex.
#[derive(Clone, Debug)]
pub enum Matcher {
    Literal(String),
    Regex(Regex),
}

impl Matcher {
    /// Build a matcher from optional search/regex spec.
    pub fn from_spec(search: &Option<String>, regex: &Option<String>) -> Result<Self> {
        if let Some(s) = search.as_ref() {
            Ok(Matcher::Literal(s.clone()))
        } else if let Some(r) = regex.as_ref() {
            Ok(Matcher::Regex(Regex::new(r).context("Invalid regex")?))
        } else {
            anyhow::bail!("Specify 'search' or 'regex'");
        }
    }
    /// Check if a line matches.
    pub fn matches(&self, line: &str) -> bool {
        match self {
            Matcher::Literal(s) => line.contains(s),
            Matcher::Regex(r) => r.is_match(line),
        }
    }
}