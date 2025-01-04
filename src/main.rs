use clap::{Parser, Subcommand};
use env_logger;
use std::{path::PathBuf, sync::Arc};

mod config;
mod error;
mod url_crawler;
mod util;
use config::Config;
use error::SpecMonkeyError;
use url_crawler::{Link, URLCrawler};
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
    CreateConfig {
        #[arg(value_name = "FILE")]
        filename: PathBuf
    }
}

fn main() -> SMResult<()> {
    // Initialize the logger. It reads the RUST_LOG environment variable to set the log level.
    env_logger::init();
    // Parse the CLI arguments.
    let cli = Cli::parse();

    // Match and handle the provided subcommand.
    match cli.command {
        Commands::CreateConfig { filename } => {
            Config::write_default(filename)?;
        }
    }
    Ok(())
}
