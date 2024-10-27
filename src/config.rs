use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub delimiter: String,
    pub extensions: Vec<String>,
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
        }
    }
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
        assert_eq!(config.extensions, vec!["rs"]);
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
}
