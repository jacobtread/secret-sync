use crate::{
    config::SecretFile,
    fs::FileSystem,
    secret::{Secret, SecretManager},
};
use eyre::Context;
use std::path::Path;

/// Download a secret file from the secret manager
pub async fn pull_secret_file<Fs: FileSystem>(
    fs: &Fs,
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

    let value: &[u8] = match &value {
        Secret::String(value) => value.as_bytes(),
        Secret::Binary(value) => value,
    };

    fs.write_file(&file_path, value).await?;

    Ok(())
}

/// Download a collection of files from the secret manager
pub async fn pull_secret_files<Fs: FileSystem>(
    fs: &Fs,
    secret: &SecretManager,
    working_path: &Path,
    files: Vec<&SecretFile>,
) -> eyre::Result<()> {
    for file in files {
        pull_secret_file(fs, secret, working_path, file).await?;
    }

    Ok(())
}
