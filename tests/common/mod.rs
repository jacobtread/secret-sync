use crate::common::aws::{test_config_base, test_container_secret_client, test_loker_container};
use std::path::Path;
use tempfile::NamedTempFile;
use testcontainers::{ContainerAsync, GenericImage};
use toml::Table;

pub mod aws;

#[allow(unused)]
pub fn normalize_test_path(path: &Path) -> String {
    path.display()
        .to_string()
        // Replace backslash windows paths to prevent breaking TOML parsing
        .replace('\\', "/")
}

#[allow(unused)]
pub async fn test_harness_aws(
    config: Table,
) -> (
    aws_sdk_secretsmanager::Client,
    NamedTempFile,
    ContainerAsync<GenericImage>,
) {
    let container = test_loker_container().await;
    let mut config_base = test_config_base(&container).await;
    let secret_manager = test_container_secret_client(&container).await;
    let config_temp_file = NamedTempFile::new().unwrap();

    // Extend the base config with the user config
    config_base.extend(config);

    let config: String = toml::to_string_pretty(&config_base).unwrap();

    tokio::fs::write(config_temp_file.path(), config)
        .await
        .unwrap();

    (secret_manager, config_temp_file, container)
}
