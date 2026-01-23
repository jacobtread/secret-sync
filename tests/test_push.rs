use crate::common::{normalize_test_path, test_harness_aws};
use assert_cmd::Command;
use aws_sdk_secretsmanager::types::Tag;
use tempfile::NamedTempFile;

mod common;

/// Tests pushing a configuration file to the secret manager
#[tokio::test]
async fn test_push_aws() {
    let temp_test_file = NamedTempFile::new().unwrap();
    let temp_test_file_path = temp_test_file.path();
    let temp_test_file_path_display = normalize_test_path(temp_test_file_path);

    let config = toml::toml! {
        [files.test-file]
        path = temp_test_file_path_display
        secret = "test-secret"
    };

    let (secret_manager, config_temp_file, _container) = test_harness_aws(config).await;

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

/// Tests pushing a configuration file to secrets manager with additional
/// metadata such as tags and description
#[tokio::test]
async fn test_push_aws_metadata() {
    let temp_test_file = NamedTempFile::new().unwrap();
    let temp_test_file_path = temp_test_file.path();
    let temp_test_file_path_display = normalize_test_path(temp_test_file_path);

    let config = toml::toml! {
        [files.test-file]
        path = temp_test_file_path_display
        secret = "test-secret"

        [files.test-file.metadata]
        description = "Example Description"
        tags = { "environment" = "Production" }
    };

    let (secret_manager, config_temp_file, _container) = test_harness_aws(config).await;

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

    let secret = secret_manager
        .describe_secret()
        .secret_id("test-secret")
        .send()
        .await
        .unwrap();

    assert_eq!(
        secret_value.secret_string().unwrap(),
        "test environment contents"
    );
    assert_eq!(secret.description().unwrap(), "Example Description");
    assert_eq!(
        secret.tags(),
        &[Tag::builder()
            .key("environment")
            .value("Production")
            .build()]
    );
}
