use clap::{Parser, Subcommand, ValueEnum};
use eyre::{Context, ContextCompat};
use serde_json::json;
use std::{env::current_dir, path::PathBuf};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{discover_nearest_config_file, read_config_file};

mod config;
mod secret;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Optional custom path to the secret-sync.toml configuration file. By default
    /// secret-sync.toml is searched for in each parent directory until discovered
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Output format to use when providing command output
    #[arg(short, long, default_value = "human")]
    pub format: OutputFormat,
}

/// Output format to use when providing program output
#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    /// Provide output in human readable format
    Human,

    /// Provide output in machine readable JSON format
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Pull the current secrets, storing the secret values
    /// in their respective files
    Pull {
        /// Optionally specify the specific secret file name
        #[arg(short, long)]
        file: Option<String>,
    },

    /// Push a secret file updating its value in the
    /// secret manage
    Push {
        /// Optionally specify the specific secret file name
        #[arg(short, long)]
        file: Option<String>,
    },
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let format = args.format.clone();

    if let Err(error) = app(args).await {
        match format {
            OutputFormat::Human => {
                return Err(error);
            }
            OutputFormat::Json => {
                tracing::error!(?error, "error occurred");

                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "error": error.to_string()
                    }))?
                );

                return Err(error);
            }
        }
    }

    Ok(())
}

async fn app(args: Args) -> eyre::Result<()> {
    // Setup colorful error logging
    color_eyre::install()?;

    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(
            EnvFilter::from_default_env()
                // Provide logging from secret-sync by default
                .add_directive("secret-sync=info".parse()?)
                //
                .add_directive("aws_sdk_secretsmanager=info".parse()?)
                .add_directive("aws_runtime=info".parse()?)
                .add_directive("aws_smithy_runtime=info".parse()?)
                .add_directive("hyper_util=info".parse()?),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_line_number(false)
                .with_target(false)
                .with_file(false)
                .with_writer(indicatif_layer.get_stderr_writer()),
        )
        .with(indicatif_layer)
        .init();

    let config_path = match args.config {
        Some(value) => value,
        None => discover_nearest_config_file().await?,
    };

    let config = read_config_file(&config_path).await?;

    match args.command {
        Commands::Pull { file } => Ok(()),
        Commands::Push { file } => Ok(()),
    }
}
