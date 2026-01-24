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

/// Tests pushing a configuration file to the secret manager
#[tokio::test]
async fn test_push_aws_glob() {
    let temp_test_file_1 = NamedTempFile::new().unwrap();
    let temp_test_file_path_1 = temp_test_file_1.path();
    let temp_test_file_path_display_1 = normalize_test_path(temp_test_file_path_1);

    let temp_test_file_2 = NamedTempFile::new().unwrap();
    let temp_test_file_path_2 = temp_test_file_2.path();
    let temp_test_file_path_display_2 = normalize_test_path(temp_test_file_path_2);

    let temp_test_file_3 = NamedTempFile::new().unwrap();
    let temp_test_file_path_3 = temp_test_file_3.path();
    let temp_test_file_path_display_3 = normalize_test_path(temp_test_file_path_3);

    let temp_test_file_4 = NamedTempFile::new().unwrap();
    let temp_test_file_path_4 = temp_test_file_4.path();
    let temp_test_file_path_display_4 = normalize_test_path(temp_test_file_path_4);

    let config = toml::toml! {
        [files.test-file-1]
        path = temp_test_file_path_display_1
        secret = "test-secret-1"

        [files.test-file-2]
        path = temp_test_file_path_display_2
        secret = "test-secret-2"

        [files.test-file-3]
        path = temp_test_file_path_display_3
        secret = "test-secret-3"

        [files.test-not-matching-file]
        path = temp_test_file_path_display_4
        secret = "test-secret-4"
    };

    let (secret_manager, config_temp_file, _container) = test_harness_aws(config).await;

    tokio::fs::write(temp_test_file_1.path(), b"test environment contents 1")
        .await
        .unwrap();

    tokio::fs::write(temp_test_file_2.path(), b"test environment contents 2")
        .await
        .unwrap();

    tokio::fs::write(temp_test_file_3.path(), b"test environment contents 3")
        .await
        .unwrap();

    tokio::fs::write(temp_test_file_4.path(), b"test environment contents 4")
        .await
        .unwrap();

    Command::new(assert_cmd::cargo_bin!())
        .arg("--config")
        .arg(config_temp_file.path().display().to_string())
        .arg("push")
        .arg("--glob")
        .arg("test-file-*")
        .assert()
        .success()
        .stdout("successfully pushed 3 secret file(s)\n");

    let secret_value = secret_manager
        .get_secret_value()
        .secret_id("test-secret-1")
        .send()
        .await
        .unwrap();

    assert_eq!(
        secret_value.secret_string.unwrap(),
        "test environment contents 1"
    );

    let secret_value = secret_manager
        .get_secret_value()
        .secret_id("test-secret-2")
        .send()
        .await
        .unwrap();

    assert_eq!(
        secret_value.secret_string.unwrap(),
        "test environment contents 2"
    );
    let secret_value = secret_manager
        .get_secret_value()
        .secret_id("test-secret-3")
        .send()
        .await
        .unwrap();

    assert_eq!(
        secret_value.secret_string.unwrap(),
        "test environment contents 3"
    );

    let secret_value = secret_manager
        .get_secret_value()
        .secret_id("test-secret-4")
        .send()
        .await
        .unwrap_err();

    // Forth secret should not have been pushed
    assert!(
        secret_value
            .into_service_error()
            .is_resource_not_found_exception()
    );
}
