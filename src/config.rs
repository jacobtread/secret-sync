//! # Config
//!
//! Configuration structures, parsing, and locating logic related
//! to configuration files.

use eyre::{Context, ContextCompat};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::current_dir,
    fmt::Debug,
    path::{Path, PathBuf},
};

/// Configuration structure for secret-sync.toml
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
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
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct BackendConfig {
    /// Provider to use
    pub provider: BackendProvider,
}

/// Provider to use for secrets
#[derive(Debug, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackendProvider {
    /// AWS (Compatible) powered backend
    #[default]
    Aws,
}

/// AWS specific configuration
#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
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
#[derive(Deserialize, PartialEq, Eq)]
pub struct AwsCredentials {
    /// AWS access key
    pub access_key_id: String,
    /// AWS access secret
    pub access_key_secret: String,
}

impl Debug for AwsCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AwsCredentials")
            .field("access_key_id", &"< REDACTED >")
            .field("access_key_secret", &"< REDACTED >")
            .finish()
    }
}

/// The secret file instance
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct SecretFile {
    /// Path relative to the config file to store the secret at
    pub path: PathBuf,
    /// Name of the secret to store / retrieve the file based on
    pub secret: String,
    /// Additional secret metadata to use when pushing secrets
    #[serde(default)]
    pub metadata: SecretMetadata,
}

/// Metadata to use with a secret file
#[derive(Debug, Deserialize, Default, PartialEq, Eq, Clone)]
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

/// Name for the secrets config file (TOML)
const CONFIG_FILE_NAME_TOML: &str = "secret-sync.toml";

/// Name for the secrets config file (JSON)
const CONFIG_FILE_NAME_JSON: &str = "secret-sync.json";

/// Searches for the nearest configuration file, checks the current
/// directory then parent directories one by one until a config is found
pub async fn discover_nearest_config_file() -> eyre::Result<PathBuf> {
    let mut path: PathBuf = current_dir().context("failed to determine current directory")?;

    loop {
        let config_path_toml = path.join(CONFIG_FILE_NAME_TOML);
        if config_path_toml.is_dir() {
            eyre::bail!("expected secret-sync.toml to be a file but got a directory");
        }

        if config_path_toml.exists() {
            return Ok(config_path_toml);
        }

        let config_path_json = path.join(CONFIG_FILE_NAME_JSON);
        if config_path_json.is_dir() {
            eyre::bail!("expected secret-sync.toml to be a file but got a directory");
        }

        if config_path_json.exists() {
            return Ok(config_path_json);
        }

        let parent = path
            .parent()
            .context("could not find secret-sync.toml in any parent directories")?;

        path = parent.to_path_buf();
    }
}

/// Parse a config file from bytes of the TOML file
fn parse_config_file_toml(file: &[u8]) -> eyre::Result<Config> {
    toml::from_slice(file).context("failed to parse config file")
}

/// Parse a config file from bytes of the JSON file
fn parse_config_file_json(file: &[u8]) -> eyre::Result<Config> {
    serde_json::from_slice(file).context("failed to parse config file")
}

/// Read a TOML config file from the provided `path`
pub async fn read_config_file(path: &Path) -> eyre::Result<Config> {
    let value = tokio::fs::read(path)
        .await
        .context("failed to read config file")?;

    let extension = match path.extension() {
        Some(value) => value.to_str().context("invalid file extension")?,

        // Assume TOML when no extension is specified
        None => return parse_config_file_toml(&value),
    };

    match extension {
        "json" => parse_config_file_json(&value),
        "toml" => parse_config_file_toml(&value),
        ext => eyre::bail!("unsupported config file extension \"{ext}\""),
    }
}

#[cfg(test)]
mod test {
    use crate::config::{parse_config_file_json, parse_config_file_toml};

    /// Tests that the example TOML configs can be parsed
    #[test]
    fn test_valid_configs_toml() {
        let configs = &[
            include_str!("../tests/samples/config/example-1.toml"),
            include_str!("../tests/samples/config/example-2.toml"),
            include_str!("../tests/samples/config/example-3.toml"),
        ];

        for config in configs {
            _ = parse_config_file_toml(config.as_bytes()).unwrap();
        }
    }

    /// Tests that the example JSON configs can be parsed
    #[test]
    fn test_valid_configs_json() {
        let configs = &[
            include_str!("../tests/samples/config/example-1.json"),
            include_str!("../tests/samples/config/example-2.json"),
            include_str!("../tests/samples/config/example-3.json"),
        ];

        for config in configs {
            _ = parse_config_file_json(config.as_bytes()).unwrap();
        }
    }

    /// Tests that the equivalent configs are both parsable and equal
    #[test]
    fn test_valid_configs_equal() {
        let configs_toml = &[
            include_str!("../tests/samples/config/example-1.toml"),
            include_str!("../tests/samples/config/example-2.toml"),
            include_str!("../tests/samples/config/example-3.toml"),
        ];

        let configs_json = &[
            include_str!("../tests/samples/config/example-1.json"),
            include_str!("../tests/samples/config/example-2.json"),
            include_str!("../tests/samples/config/example-3.json"),
        ];

        for (config_toml, config_json) in configs_toml.iter().zip(configs_json.iter()) {
            let config_toml = parse_config_file_toml(config_toml.as_bytes()).unwrap();
            let config_json = parse_config_file_json(config_json.as_bytes()).unwrap();
            assert_eq!(config_toml, config_json);
        }
    }
}
