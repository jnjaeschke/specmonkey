use log::info;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::SMResult;

/// Struct representing the configuration parsed from the YAML file.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    pub(super) extensions: Vec<String>,
    pub(super) domains: Vec<String>,
    pub(super) source_repository: Repository,
    pub(super) index_repository: Repository,
}

/// Struct representing a repository with a URL and branch.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Repository {
    url: String,
    branch: String,
}

impl Config {
    /// Parses a YAML file at the given path into a `Config` struct.
    pub fn try_from_file<P: AsRef<Path>>(file_path: P) -> SMResult<Config> {
        // Open the YAML file
        let mut file = File::open(&file_path)?;

        // Read the file contents into a string
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Deserialize the YAML contents into the Config struct
        let config: Config = serde_yaml::from_str(&contents)?;
        info!("Parsed config file '{}'", &file_path.as_ref().display());

        Ok(config)
    }

    pub fn write_default<P: AsRef<Path>>(file_path: P) -> SMResult<()> {
        let yaml_string = serde_yaml::to_string(&Self::default())?;
        std::fs::write(&file_path, yaml_string)?;
        info!(
            "Wrote default config yaml file to '{}'",
            file_path.as_ref().display()
        );
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            extensions: vec![String::from("h"), String::from("cpp")],
            domains: vec![String::from("example.com")],
            source_repository: Default::default(),
            index_repository: Default::default(),
        }
    }
}

impl Default for Repository {
    fn default() -> Self {
        Repository {
            url: String::from("https://github.com/org/repo"),
            branch: String::from("main"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_parse_yaml_config() -> SMResult<()> {
        // Create a temporary YAML file
        let mut temp_file = NamedTempFile::new()?;
        let yaml_content = r#"
extensions:
  - ".js"
  - ".ts"
domains:
  - "example.com"
  - "spec.org"
source_repository:
  url: "https://github.com/user/source-repo"
  branch: "main"
index_repository:
  url: "https://github.com/user/index-repo"
  branch: "develop"
"#;
        write!(temp_file, "{}", yaml_content)?;

        // Parse the YAML configuration
        let config = Config::try_from_file(temp_file.path())?;

        // Assert the parsed content
        assert_eq!(config.extensions, vec![".js", ".ts"]);
        assert_eq!(config.domains, vec!["example.com", "spec.org"]);
        assert_eq!(
            config.source_repository.url,
            "https://github.com/user/source-repo"
        );
        assert_eq!(config.source_repository.branch, "main");
        assert_eq!(
            config.index_repository.url,
            "https://github.com/user/index-repo"
        );
        assert_eq!(config.index_repository.branch, "develop");

        Ok(())
    }

    #[test]
    fn test_parse_yaml_config_invalid_path() {
        let result = Config::try_from_file("nonexistent.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yaml_config_invalid_yaml() -> SMResult<()> {
        // Create a temporary YAML file with invalid content
        let mut temp_file = NamedTempFile::new()?;
        let invalid_yaml = r#"
extensions:
  - ".js"
  - ".ts"
domains: "example.com"  # Should be a list
source_repository:
  url: "https://github.com/user/source-repo"
  branch: "main"
index_repository:
  url: "https://github.com/user/index-repo"
  branch: "develop"
"#;
        write!(temp_file, "{}", invalid_yaml)?;

        // Attempt to parse the invalid YAML configuration
        let result = Config::try_from_file(temp_file.path());
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_write_default() -> SMResult<()> {
        let tempdir = TempDir::new()?;
        let config_file = tempdir.path().with_file_name("config.yaml");
        Config::write_default(&config_file)?;
        let config = Config::try_from_file(config_file)?;
        assert_eq!(config, Config::default());
        Ok(())
    }
}
