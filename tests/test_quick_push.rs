use crate::common::{normalize_test_path, test_harness_aws};
use assert_cmd::Command;
use tempfile::NamedTempFile;

mod common;

/// Tests quick pushing a configuration file to the secret manager
#[tokio::test]
async fn test_quick_push_aws() {
    let temp_test_file = NamedTempFile::new().unwrap();
    let temp_test_file_path = temp_test_file.path();
    let temp_test_file_path_display = normalize_test_path(temp_test_file_path);
    let (secret_manager, config_temp_file, _container) = test_harness_aws(toml::Table::default()).await;

    tokio::fs::write(temp_test_file.path(), b"test environment contents")
        .await
        .unwrap();

    Command::new(assert_cmd::cargo_bin!())
        .arg("--config")
        .arg(config_temp_file.path().display().to_string())
        .arg("quick-push")
        .arg("--path")
        .arg(temp_test_file_path_display)
        .arg("--secret")
        .arg("test-secret")
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
