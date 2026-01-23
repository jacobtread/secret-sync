use crate::common::{normalize_test_path, test_harness_aws};
use assert_cmd::Command;
use tempfile::NamedTempFile;

mod common;

/// Tests pulling a configuration file from a AWS secret manager
#[tokio::test]
async fn test_pull_aws() {
    let temp_test_file = NamedTempFile::new().unwrap();
    let temp_test_file_path = temp_test_file.path();
    let temp_test_file_path_display = normalize_test_path(temp_test_file_path);

    let config = toml::toml! {
        [files.test-file]
        path = temp_test_file_path_display
        secret = "test-secret"
    };

    let (secret_manager, config_temp_file, _container) = test_harness_aws(config).await;

    secret_manager
        .create_secret()
        .name("test-secret")
        .secret_string("test environment contents")
        .send()
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

/// Tests pulling a unknown configuration file from AWS secret manager
#[tokio::test]
async fn test_pull_aws_unknown() {
    let temp_test_file = NamedTempFile::new().unwrap();
    let temp_test_file_path = temp_test_file.path();
    let temp_test_file_path_display = normalize_test_path(temp_test_file_path);

    let config = toml::toml! {
        [files.test-file]
        path = temp_test_file_path_display
        secret = "test-secret"
    };

    let (_secret_manager, config_temp_file, _container) = test_harness_aws(config).await;

    Command::new(assert_cmd::cargo_bin!())
        .arg("--disable-color")
        .arg("--format")
        .arg("json")
        .arg("--config")
        .arg(config_temp_file.path().display().to_string())
        .arg("pull")
        .assert()
        .failure()
        .stdout("{\"error\":\"secret \\\"test-secret\\\" not found\",\"success\":false}\n");
}
