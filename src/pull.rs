use crate::{
    config::SecretFile,
    secret::{Secret, SecretManager},
};
use eyre::{Context, ContextCompat};
use std::path::Path;
use tokio::fs::create_dir_all;

/// Download a secret file from the secret manager
pub async fn pull_secret_file(
    secret: &SecretManager,
    working_path: &Path,
    file: &SecretFile,
) -> eyre::Result<()> {
    let value = secret
        .get_secret(&file.secret)
        .await
        .context("failed to retrieve secret")?;

    let file_path = if file.path.is_absolute() {
        file.path.clone()
    } else {
        working_path.join(file.path.clone())
    };

    let parent_path = file_path
        .parent()
        .context("file parent path does not exist")?;

    if !parent_path.exists() {
        tracing::debug!(
            ?file_path,
            ?parent_path,
            "does not exist, creating parent path for secret file"
        );

        create_dir_all(parent_path)
            .await
            .context("failed to create parent directory for secret file")?;
    }

    let value: &[u8] = match &value {
        Secret::String(value) => value.as_bytes(),
        Secret::Binary(value) => value,
    };

    tokio::fs::write(file_path, value)
        .await
        .context("failed to write secret to file")?;

    Ok(())
}

/// Download a collection of files from the secret manager
pub async fn pull_secret_files(
    secret: &SecretManager,
    working_path: &Path,
    files: Vec<&SecretFile>,
) -> eyre::Result<()> {
    for file in files {
        pull_secret_file(secret, working_path, file).await?;
    }

    Ok(())
}
