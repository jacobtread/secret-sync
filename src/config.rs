use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use eyre::{Context, ContextCompat};
use serde::Deserialize;

/// Configuration structure for secret-sync.toml
#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// AWS specific configuration
    pub aws: AwsConfig,
    /// The secret files to operate on
    pub files: Vec<SecretFile>,
}

/// AWS specific configuration
#[derive(Deserialize, Default)]
pub struct AwsConfig {
    /// AWS profile to use the sdk with
    pub profile: Option<String>,
}

/// The secret file instance
#[derive(Deserialize)]
pub struct SecretFile {
    /// Path relative to the config file to store the secret at
    pub path: PathBuf,
    /// Name of the secret to store / retrieve the file based on
    pub secret: String,
}

const CONFIG_FILE_NAME: &str = "secret-sync.toml";

/// Searches for the nearest configuration file, checks the current
/// directory then parent directories one by one until a config is found
pub async fn discover_nearest_config_file() -> eyre::Result<PathBuf> {
    let mut path: PathBuf = current_dir().context("failed to determine current directory")?;

    loop {
        let config_path = path.join(CONFIG_FILE_NAME);

        if config_path.is_dir() {
            eyre::bail!("expected secret-sync.toml to be a file but got a directory");
        }

        if config_path.exists() {
            return Ok(config_path);
        }

        let parent = path
            .parent()
            .context("could not find secret-sync.toml in any parent directories")?;

        path = parent.to_path_buf();
    }
}

pub fn parse_config_file(file: &[u8]) -> eyre::Result<Config> {
    serde_json::from_slice(file).context("failed to parse config file")
}

pub async fn read_config_file(file: &Path) -> eyre::Result<Config> {
    let value = tokio::fs::read(file)
        .await
        .context("failed to read config file")?;
    parse_config_file(&value)
}
