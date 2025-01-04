use clap::{Parser, Subcommand};
use env_logger;
use std::{path::PathBuf, sync::Arc};

mod config;
mod error;
mod index;
mod url_crawler;
mod util;
use config::Config;
use error::SpecMonkeyError;
use index::Index;
use url_crawler::URLCrawler;
use util::gather_files;

pub type SMResult<T> = Result<T, SpecMonkeyError>;

#[derive(Parser)]
#[command(name = "specmonkey")]
#[command(about = "Index references in repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Index references from the source repository to the index repository.
    Index {
        /// Path to the configuration YAML file.
        #[arg(short, long, value_name = "FILE")]
        config_file: PathBuf,

        /// Path to the source repository.
        #[arg(value_name = "SOURCE_REPO")]
        source_repository_path: PathBuf,

        /// Path to the index repository.
        #[arg(value_name = "INDEX_REPO")]
        index_repository_path: PathBuf,
    },
    CreateConfig {
        #[arg(value_name = "FILE")]
        filename: PathBuf,
    },
}

fn main() -> SMResult<()> {
    // Initialize the logger. It reads the RUST_LOG environment variable to set the log level.
    env_logger::init();
    // Parse the CLI arguments.
    let cli = Cli::parse();

    // Match and handle the provided subcommand.
    match cli.command {
        Commands::Index {
            config_file,
            source_repository_path,
            index_repository_path,
        } => {
            let Config {
                extensions,
                domains,
                source_repository,
                index_repository,
            } = Config::try_from_file(&config_file)?;

            // todo: pull git repo

            // scan repo for urls
            let raw_url_data =
                gather_files(&source_repository_path, Arc::new(extensions)).map(|filepaths| {
                    URLCrawler::find_urls(filepaths, &source_repository_path, domains)
                })?;

            Index::from_raw_data(raw_url_data).write_json(index_repository_path)?;

            // commit + push the index repository
        }
        Commands::CreateConfig { filename } => {
            Config::write_default(filename)?;
        }
    }
    Ok(())
}
