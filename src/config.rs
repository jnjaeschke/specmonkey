use log::info;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{self, File};
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
        // Open the JSON file
        let mut file = File::open(&file_path)?;

        // Read the file contents into a string
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Deserialize the YAML contents into the Config struct
        let config: Config = serde_json::from_str(&contents)?;
        info!("Parsed config file '{}'", &file_path.as_ref().display());

        Ok(config)
    }

    pub fn write_default<P: AsRef<Path>>(file_path: P) -> SMResult<()> {
        // Open the file in write mode, creating it if it doesn't exist.
        let file = fs::File::create(&file_path)?;

        // Serialize the Vec<IndexItem> to JSON and write it to the file.
        serde_json::to_writer_pretty(file, &Self::default())?;
        info!(
            "Wrote default config json file to '{}'",
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
    use tempfile::TempDir;

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
