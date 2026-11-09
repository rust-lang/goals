use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use indexmap::IndexMap;
use regex::Regex;
use serde::Deserialize;
use spanned::{Context as _, Result};

#[derive(Deserialize)]
pub struct Configuration {
    /// Defines the valid "asks" of teams. The key is the ask, the value is an extended description.
    /// IndexMap is used to preserve the ordering as defined in the TOML file.
    pub team_asks: IndexMap<String, TeamAskDetails>,
}

#[derive(Deserialize)]
pub struct TeamAskDetails {
    /// A short descriptor of the team ask suitable for inclusion in a table
    pub short: String,

    /// Longer description
    pub about: String,

    /// If true, do not include in the RFC tables.
    #[serde(default)]
    pub elide: bool,
}

impl Configuration {
    pub fn get() -> &'static Configuration {
        lazy_static::lazy_static! {
            static ref CONFIG: Configuration = Configuration::load().unwrap();
        }
        &CONFIG
    }

    fn load() -> Result<Configuration> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let toml_file = manifest_dir.join("../../rust-project-goals.toml");
        let toml_string = std::fs::read_to_string(&toml_file)
            .with_path_context(&toml_file, "loading configuration")?;
        Ok(toml::from_str(&toml_string)?)
    }
}

// Goals-specific configuration for markdown processing
// This is separate from the main Configuration above

#[derive(Deserialize, Debug, Default)]
struct TomlBookConfig {
    #[serde(default)]
    preprocessor: TomlPreprocessorConfig,
}

#[derive(Deserialize, Debug, Default)]
struct TomlPreprocessorConfig {
    #[serde(default)]
    goals: TomlGoalsConfig,
}

#[derive(Deserialize, Debug, Default, Clone)]
struct TomlGoalsConfig {
    /// Static link definitions (name -> URL)
    /// Maps from link names like "Help wanted" to URLs like "https://img.shields.io/badge/Help%20wanted-yellow"
    #[serde(default)]
    pub links: HashMap<String, String>,

    /// Linkifier patterns (regex pattern -> URL template)
    /// Maps from patterns like "RFC #([0-9]+)" to URL templates like "https://github.com/rust-lang/rfcs/pull/$1"
    #[serde(default)]
    pub linkifiers: HashMap<String, String>,

    /// User display name overrides (username -> display name)
    /// Maps from usernames like "@nikomatsakis" to display names like "Niko Matsakis"
    #[serde(default)]
    pub users: HashMap<String, String>,

    /// Usernames to ignore during auto-linking
    /// List of usernames like ["@bot", "@automated"] that should not be auto-linked
    #[serde(default)]
    pub ignore_users: Vec<String>,
}

/// Parsed and processed goals configuration ready for use
/// This is the public interface that components should use
#[derive(Debug, Clone)]
pub struct GoalsConfig {
    /// Static link definitions (name -> URL)
    pub links: HashMap<String, String>,
    /// Compiled linkifier patterns (regex -> URL template)
    pub linkifiers: Vec<(Regex, String)>,
    /// User display name overrides (username -> display name)
    pub users: HashMap<String, String>,
    /// Usernames to ignore during auto-linking
    pub ignore_users: Vec<String>,
}

impl GoalsConfig {
    /// Load and parse goals configuration from book.toml
    pub fn from_book_toml(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).context(format!(
            "Failed to read book.toml at {}",
            path.as_ref().display()
        ))?;
        let book_config: TomlBookConfig = toml::from_str(&content).context(format!(
            "Failed to parse book.toml at {}",
            path.as_ref().display()
        ))?;
        Self::from_toml_goals_config(book_config.preprocessor.goals)
    }

    /// Convert from raw TomlGoalsConfig to processed GoalsConfig
    fn from_toml_goals_config(config: TomlGoalsConfig) -> anyhow::Result<Self> {
        // Compile linkifier regex patterns
        let linkifiers: Vec<(Regex, String)> = config
            .linkifiers
            .into_iter()
            .map(|(pattern, url_template)| {
                let regex = Regex::new(&format!(r"\[{}\]", pattern))
                    .context(format!("Invalid linkifier pattern: {}", pattern))?;
                Ok((regex, url_template))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(GoalsConfig {
            links: config.links,
            linkifiers,
            users: config.users,
            ignore_users: config.ignore_users,
        })
    }

    /// Create empty/default configuration
    pub fn default() -> Self {
        Self {
            links: HashMap::new(),
            linkifiers: Vec::new(),
            users: HashMap::new(),
            ignore_users: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_goals_config_empty_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# empty book.toml").unwrap();

        let config = GoalsConfig::from_book_toml(file.path()).unwrap();
        assert!(config.links.is_empty());
        assert!(config.linkifiers.is_empty());
        assert!(config.users.is_empty());
        assert!(config.ignore_users.is_empty());
    }

    #[test]
    fn test_goals_config_complete_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[preprocessor.goals.links]
"Help wanted" = "https://img.shields.io/badge/Help%20wanted-yellow"
"Complete" = "https://img.shields.io/badge/Complete-green"

[preprocessor.goals.linkifiers]
"RFC #([0-9]+)" = "https://github.com/rust-lang/rfcs/pull/$1"
"([a-zA-Z0-9-]+)/([a-zA-Z0-9-]+)#([0-9]+)" = "https://github.com/$1/$2/issues/$3"

[preprocessor.goals.users]
"@nikomatsakis" = "Niko Matsakis"
"@Nadrieril" = "@Nadrieril"

[preprocessor.goals]
ignore_users = ["@bot", "@automated"]
        "#
        )
        .unwrap();

        let config = GoalsConfig::from_book_toml(file.path()).unwrap();

        // Check links
        assert_eq!(config.links.len(), 2);
        assert_eq!(
            config.links.get("Help wanted"),
            Some(&"https://img.shields.io/badge/Help%20wanted-yellow".to_string())
        );
        assert_eq!(
            config.links.get("Complete"),
            Some(&"https://img.shields.io/badge/Complete-green".to_string())
        );

        // Check linkifiers (now compiled regex patterns)
        assert_eq!(config.linkifiers.len(), 2);
        // Check that the patterns are compiled correctly by finding one with the expected URL template
        let rfc_linkifier = config
            .linkifiers
            .iter()
            .find(|(_, template)| template == "https://github.com/rust-lang/rfcs/pull/$1");
        assert!(rfc_linkifier.is_some());

        // Check users
        assert_eq!(config.users.len(), 2);
        assert_eq!(
            config.users.get("@nikomatsakis"),
            Some(&"Niko Matsakis".to_string())
        );
        assert_eq!(
            config.users.get("@Nadrieril"),
            Some(&"@Nadrieril".to_string())
        );

        // Check ignore_users
        assert_eq!(config.ignore_users.len(), 2);
        assert!(config.ignore_users.contains(&"@bot".to_string()));
        assert!(config.ignore_users.contains(&"@automated".to_string()));
    }

    #[test]
    fn test_goals_config_partial_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[book]
title = "Some Book"

[preprocessor.goals.users]
"@alice" = "Alice"

[preprocessor.other]
some_other_config = "value"
        "#
        )
        .unwrap();

        let config = GoalsConfig::from_book_toml(file.path()).unwrap();

        // Only users should be populated, others should be empty
        assert!(config.links.is_empty());
        assert!(config.linkifiers.is_empty());
        assert_eq!(config.users.len(), 1);
        assert_eq!(config.users.get("@alice"), Some(&"Alice".to_string()));
        assert!(config.ignore_users.is_empty());
    }

    #[test]
    fn test_goals_config_invalid_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "invalid toml [").unwrap();

        let result = GoalsConfig::from_book_toml(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_goals_config_missing_file() {
        let result = GoalsConfig::from_book_toml("/nonexistent/file.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_goals_config_default() {
        let config = GoalsConfig::default();
        assert!(config.links.is_empty());
        assert!(config.linkifiers.is_empty());
        assert!(config.users.is_empty());
        assert!(config.ignore_users.is_empty());
    }

    #[test]
    fn test_goals_config_clone() {
        let mut config = GoalsConfig::default();
        config
            .users
            .insert("@test".to_string(), "Test User".to_string());

        let cloned = config.clone();
        assert_eq!(config.users, cloned.users);
    }
}
