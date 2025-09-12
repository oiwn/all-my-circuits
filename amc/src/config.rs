use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub delimiter: String,
    pub extensions: Vec<String>,
    #[serde(default = "default_llm_prompt")]
    pub llm_prompt: String,
    #[serde(default)]
    pub excluded_folders: Vec<String>,
}

impl Config {
    /// Load configuration from the specified file path, falling back to default if the file doesn't exist
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if path.as_ref().exists() {
            Self::from_file(path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from the specified file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_content = fs::read_to_string(&path).context(format!(
            "Failed to read config file: {}",
            path.as_ref().display()
        ))?;

        Self::from_str(&config_content)
    }

    /// Parse configuration from a string
    pub fn from_str(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to parse config")
    }

    /// Create a default configuration
    pub fn default() -> Self {
        Self {
            delimiter: "---".to_string(),
            extensions: vec!["rs".to_string()],
            llm_prompt: default_llm_prompt(),
            excluded_folders: Vec::new(),
        }
    }

    /// Generate default configuration as TOML string
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize config to TOML")
    }

    /// Write default configuration to file
    pub fn write_default<P: AsRef<Path>>(path: P) -> Result<()> {
        let default_config = Self::default();
        let toml_content = default_config.to_toml()?;

        fs::write(&path, toml_content).context(format!(
            "Failed to write config file: {}",
            path.as_ref().display()
        ))?;

        Ok(())
    }
}

fn default_llm_prompt() -> String {
    r#"
This is a concatenated source code file containing multiple source files from a project.
Each file section begins and ends with a delimiter line "---".
After the opening delimiter, there is metadata about the file:
- File: relative path to the source file
- Last commit: Git commit hash of the last change
- Last update: Unix timestamp of the last change

Please analyze the code with these aspects in mind:
1. The relationship and dependencies between files
2. The overall architecture and design patterns used
3. Any potential improvements or issues you notice
4. Consider the context of changes based on the Git metadata

The code sections follow below:
"#.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() -> Result<()> {
        let config_content = r#"
            delimiter = "---"
            extensions = ["rs", "toml"]
        "#;

        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", config_content)?;

        let config = Config::from_file(temp_file.path())?;
        assert_eq!(config.delimiter, "---");
        assert_eq!(config.extensions, vec!["rs", "toml"]);

        Ok(())
    }

    #[test]
    fn test_load_nonexistent_config() -> Result<()> {
        let config = Config::load("nonexistent-config.toml")?;
        assert_eq!(config.delimiter, "---");
        assert!(config.extensions.contains(&"rs".to_string()));
        assert!(config.llm_prompt.contains("concatenated source code"));
        Ok(())
    }

    #[test]
    fn test_parse_config_from_str() -> Result<()> {
        let config_content = r#"
            delimiter = "==="
            extensions = ["txt"]
        "#;

        let config = Config::from_str(config_content)?;
        assert_eq!(config.delimiter, "===");
        assert_eq!(config.extensions, vec!["txt"]);

        Ok(())
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.delimiter, "---");
        assert_eq!(config.extensions, vec!["rs"]);
    }

    #[test]
    fn test_custom_prompt_config() -> Result<()> {
        let config_content = r#"
            delimiter = "---"
            extensions = ["rs"]
            llm_prompt = "Custom prompt for analysis"
        "#;

        let config = Config::from_str(config_content)?;
        assert_eq!(config.llm_prompt, "Custom prompt for analysis");
        Ok(())
    }
}
