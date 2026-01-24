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

/// Tests pulling a configuration file from a AWS secret manager using
/// a glob for matching which files to pull
#[tokio::test]
async fn test_pull_aws_glob() {
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

    secret_manager
        .create_secret()
        .name("test-secret-1")
        .secret_string("test environment contents 1")
        .send()
        .await
        .unwrap();

    secret_manager
        .create_secret()
        .name("test-secret-2")
        .secret_string("test environment contents 2")
        .send()
        .await
        .unwrap();

    secret_manager
        .create_secret()
        .name("test-secret-3")
        .secret_string("test environment contents 3")
        .send()
        .await
        .unwrap();

    Command::new(assert_cmd::cargo_bin!())
        .arg("--config")
        .arg(config_temp_file.path().display().to_string())
        .arg("pull")
        .arg("--glob")
        .arg("test-file-*")
        .assert()
        .success()
        .stdout("successfully pulled 3 secret file(s)\n");

    let file_data = tokio::fs::read(temp_test_file_path_1).await.unwrap();
    assert_eq!(file_data, b"test environment contents 1");
    let file_data = tokio::fs::read(temp_test_file_path_2).await.unwrap();
    assert_eq!(file_data, b"test environment contents 2");
    let file_data = tokio::fs::read(temp_test_file_path_3).await.unwrap();
    assert_eq!(file_data, b"test environment contents 3");
}
