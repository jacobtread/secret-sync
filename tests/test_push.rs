use assert_cmd::Command;
use tempfile::NamedTempFile;

use crate::common::aws::{test_config_base, test_container_secret_client, test_loker_container};

mod common;

/// Tests pulling a configuration file from
#[tokio::test]
async fn test_push_aws() {
    let container = test_loker_container().await;
    let config_base = test_config_base(&container).await;
    let secret_manager = test_container_secret_client(&container).await;

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

    tokio::fs::write(temp_test_file.path(), b"test environment contents")
        .await
        .unwrap();

    Command::new(assert_cmd::cargo_bin!())
        .arg("--config")
        .arg(config_temp_file.path().display().to_string())
        .arg("push")
        .assert()
        .success()
        .stdout("successfully pushed 1 secret file(s)\n");

    let secret_value = secret_manager
        .get_secret_value()
        .secret_id("test-secret")
        .send()
        .await
        .unwrap();

    assert_eq!(
        secret_value.secret_string.unwrap(),
        "test environment contents"
    );
}
