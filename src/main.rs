use crate::{
    config::{BackendProvider, SecretFile, discover_nearest_config_file, read_config_file},
    fs::real::RealFs,
    pull::pull_secret_files,
    push::push_secret_files,
    secret::aws::AwsSecretManager,
};
use clap::{Parser, Subcommand, ValueEnum};
use eyre::{Context, ContextCompat};
use serde_json::json;
use std::{
    collections::HashMap,
    path::{PathBuf, absolute},
};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod fs;
mod pull;
mod push;
mod secret;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Optional custom path to the secret-sync.toml configuration file. By default
    /// secret-sync.toml (and secret-sync.json) is searched for in each parent
    /// directory until discovered
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Output format to use when providing command output
    #[arg(short, long, default_value = "human")]
    pub format: OutputFormat,

    /// Disable color in the output
    #[arg(short, long, default_value_t = false)]
    pub disable_color: bool,
}

/// Output format to use when providing program output
#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    /// Provide output in human readable format
    Human,

    /// Provide output in machine readable JSON format
    Json,
}

#[derive(clap::Args, Clone)]
pub struct TargetFilter {
    /// Optionally specify file names to match
    #[arg(short, long)]
    file: Option<Vec<String>>,

    /// Optionally specify globs for file names to match
    #[arg(short, long)]
    glob: Option<Vec<String>>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Pull the current secrets, storing the secret values
    /// in their respective files
    Pull {
        #[command(flatten)]
        filter: TargetFilter,
    },

    /// Push a secret file updating its value in the
    /// secret manage
    Push {
        #[command(flatten)]
        filter: TargetFilter,
    },
}

/// Output data for a successful run
struct Output {
    /// Text version
    text: String,
    /// JSON version
    json: serde_json::Value,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let format = args.format.clone();

    match app(args).await {
        Ok(output) => match format {
            OutputFormat::Human => {
                println!("{}", output.text);
            }
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&output.json)?);
            }
        },
        Err(error) => match format {
            OutputFormat::Human => {
                return Err(error);
            }
            OutputFormat::Json => {
                tracing::error!(?error, "error occurred");

                println!(
                    "{}",
                    serde_json::to_string(&json!({
                        "success": false,
                        "error": error.to_string()
                    }))?
                );

                return Err(error);
            }
        },
    }

    Ok(())
}

/// Initialize the logging and indicator layers
fn init_logging() -> eyre::Result<()> {
    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(
            EnvFilter::from_default_env()
                // Provide logging from secret-sync by default
                .add_directive("secret_sync=info".parse()?)
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

    Ok(())
}

/// Main logic entrypoint
async fn app(args: Args) -> eyre::Result<Output> {
    if !args.disable_color {
        // Setup colorful error logging
        color_eyre::install()?;
    }

    init_logging()?;

    let config_path = match args.config {
        Some(value) => value,
        None => discover_nearest_config_file().await?,
    };

    let config_path = absolute(config_path).context("failed to get absolute config path")?;

    tracing::debug!(?config_path, "found config file");

    let config = read_config_file(&config_path).await?;

    let secret = match config.backend.provider {
        BackendProvider::Aws => Box::new(AwsSecretManager::from_config(&config.aws).await?),
    };

    let working_path = config_path
        .parent()
        .context("missing config parent path unable to use directory for context")?;

    tracing::debug!(?working_path, "working path");

    let fs = RealFs;

    match args.command {
        Commands::Pull { filter } => {
            let files = filter_files(&config.files, &filter);

            if files.is_empty() && !config.files.is_empty() {
                eyre::bail!(
                    "no files matching filter within \"{}\"",
                    config_path.display()
                )
            }

            let total_files = files.len();
            pull_secret_files(&fs, secret.as_ref(), working_path, files).await?;

            Ok(Output {
                text: format!("successfully pulled {} secret file(s)", total_files),
                json: json!({
                    "success": true
                }),
            })
        }
        Commands::Push { filter } => {
            let files = filter_files(&config.files, &filter);

            if files.is_empty() && !config.files.is_empty() {
                eyre::bail!(
                    "no files matching filter within \"{}\"",
                    config_path.display()
                )
            }

            let total_files = files.len();
            push_secret_files(&fs, secret.as_ref(), working_path, files).await?;

            Ok(Output {
                text: format!("successfully pushed {} secret file(s)", total_files),
                json: json!({
                    "success": true
                }),
            })
        }
    }
}

/// Filter a set of `files` only returning the results that match `filter`
fn filter_files<'a>(
    files: &'a HashMap<String, SecretFile>,
    filter: &TargetFilter,
) -> Vec<&'a SecretFile> {
    files
        .iter()
        .filter(|(name, _file)| {
            // Nothing to filter against
            if filter.file.is_none() && filter.glob.is_none() {
                return true;
            }

            let name_matches = filter
                .file
                .as_ref()
                .is_some_and(|file_names| file_names.contains(name));

            let glob_matches = filter.glob.as_ref().is_some_and(|globs| {
                globs
                    .iter()
                    .any(|glob| fast_glob::glob_match(glob.as_bytes(), name.as_bytes()))
            });

            name_matches || glob_matches
        })
        .map(|(_key, value)| value)
        .collect()
}
