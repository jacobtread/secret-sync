use std::{
    collections::HashMap,
    env::current_dir,
    path::{Path, PathBuf},
};

use eyre::{Context, ContextCompat};
use serde::Deserialize;

/// Configuration structure for secret-sync.toml
#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Config deciding which backend to use
    pub backend: BackendConfig,
    /// AWS specific configuration
    pub aws: AwsConfig,
    /// The secret files to operate on
    pub files: HashMap<String, SecretFile>,
}

/// Config around the secrets backend to use
#[derive(Deserialize, Default)]
#[serde(default)]
pub struct BackendConfig {
    pub provider: BackendProvider,
}

/// Provider to use for secrets
#[derive(Deserialize, Default, Clone, Copy)]
pub enum BackendProvider {
    #[default]
    Aws,
}

/// AWS specific configuration
#[derive(Deserialize, Default)]
pub struct AwsConfig {
    /// AWS profile to use the sdk with
    pub profile: Option<String>,

    /// Optionally override the AWS region
    pub region: Option<String>,

    /// Custom override for the AWS secret manager endpoint
    pub endpoint: Option<String>,

    /// Custom AWS credentials to use
    pub credentials: Option<AwsCredentials>,
}

/// AWS credentials
#[derive(Deserialize)]
pub struct AwsCredentials {
    /// AWS access key
    pub access_key_id: String,
    /// AWS access secret
    pub access_key_secret: String,
}

/// The secret file instance
#[derive(Deserialize)]
pub struct SecretFile {
    /// Path relative to the config file to store the secret at
    pub path: PathBuf,
    /// Name of the secret to store / retrieve the file based on
    pub secret: String,
    /// Additional secret metadata to use when pushing secrets
    #[serde(default)]
    pub metadata: SecretMetadata,
}

#[derive(Deserialize, Default, Debug, PartialEq, Eq, Clone)]
#[serde(default)]
pub struct SecretMetadata {
    /// Optional description of the secret, this will be attached
    /// to the secret if using the AWS backend
    ///
    /// Will only be used on the first creation push
    pub description: Option<String>,

    /// Optional tags to attach to the secret (AWS Backend)
    ///
    /// Will only be used on the first creation push
    pub tags: Option<HashMap<String, String>>,
}

/// Name for the secrets config file
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

/// Parse a config file from bytes of the TOML file
fn parse_config_file(file: &[u8]) -> eyre::Result<Config> {
    toml::from_slice(file).context("failed to parse config file")
}

/// Read a TOML config file from the provided `path`
pub async fn read_config_file(path: &Path) -> eyre::Result<Config> {
    let value = tokio::fs::read(path)
        .await
        .context("failed to read config file")?;
    parse_config_file(&value)
}
