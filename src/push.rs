use crate::{
    config::SecretFile,
    secret::{Secret, SecretManager},
};
use eyre::Context;
use std::path::Path;

/// Upload a secret file to the secret manager
pub async fn push_secret_file(
    secret: &SecretManager,
    working_path: &Path,
    file: &SecretFile,
) -> eyre::Result<()> {
    let file_path = if file.path.is_absolute() {
        file.path.clone()
    } else {
        working_path.join(file.path.clone())
    };

    if !file_path.exists() {
        eyre::bail!("cannot push secret, file does not exist");
    }

    let value = tokio::fs::read(&file_path)
        .await
        .context("failed to read secret file")?;

    let value = match String::from_utf8(value) {
        Ok(value) => Secret::String(value),
        Err(error) => Secret::Binary(error.into_bytes()),
    };

    secret
        .set_secret(&file.secret, value, &file.metadata)
        .await
        .context("failed to store secret")?;

    Ok(())
}

/// Upload a collection of secret files to the secret manager
pub async fn push_secret_files(
    secret: &SecretManager,
    working_path: &Path,
    files: Vec<&SecretFile>,
) -> eyre::Result<()> {
    for file in files {
        push_secret_file(secret, working_path, file).await?;
    }

    Ok(())
}
