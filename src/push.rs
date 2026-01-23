use crate::{
    config::SecretFile,
    fs::FileSystem,
    secret::{Secret, SecretManager},
};
use eyre::Context;
use std::path::Path;

/// Upload a secret file to the secret manager
pub async fn push_secret_file<Fs: FileSystem>(
    fs: &Fs,
    secret: &dyn SecretManager,
    working_path: &Path,
    file: &SecretFile,
) -> eyre::Result<()> {
    let file_path = if file.path.is_absolute() {
        file.path.clone()
    } else {
        working_path.join(file.path.clone())
    };

    let value = fs.read_file(&file_path).await?;

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
pub async fn push_secret_files<Fs: FileSystem>(
    fs: &Fs,
    secret: &dyn SecretManager,
    working_path: &Path,
    files: Vec<&SecretFile>,
) -> eyre::Result<()> {
    for file in files {
        push_secret_file(fs, secret, working_path, file).await?;
    }

    Ok(())
}
