use assert_cmd::Command;
use tempfile::NamedTempFile;

use crate::common::aws::{test_config_base, test_container_secret_client, test_loker_container};

mod common;

/// Tests pulling a configuration file from
#[tokio::test]
async fn test_pull_aws() {
    let container = test_loker_container().await;
    let config_base = test_config_base(&container).await;
    let secret_manager = test_container_secret_client(&container).await;

    secret_manager
        .create_secret()
        .name("test-secret")
        .secret_string("test environment contents")
        .send()
        .await
        .unwrap();

    let config_temp_file = NamedTempFile::new().unwrap();

    let temp_test_file = NamedTempFile::new().unwrap();
    let temp_test_file_path = temp_test_file.path();
    let temp_test_file_path_display = temp_test_file_path
        .display()
        .to_string()
        // Replace backslash windows paths to prevent breaking TOML parsing
        .replace('\\', "/");

    let config = format!(
        r#"
{config_base}

[files.test-file]
path = "{temp_test_file_path_display}"
secret = "test-secret"
        "#
    );

    tokio::fs::write(config_temp_file.path(), config)
        .await
        .unwrap();

    Command::new(assert_cmd::cargo_bin!())
        .arg("--config")
        .arg(config_temp_file.path().display().to_string())
        .arg("pull")
        .assert()
        .success()
        .stdout("successfully pulled 1 secret file(s)\n");

    let file_data = tokio::fs::read(temp_test_file_path).await.unwrap();

    assert_eq!(file_data, b"test environment contents");
}
