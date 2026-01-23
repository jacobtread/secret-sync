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

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use mockall::predicate::eq;

    use crate::{
        config::{SecretFile, SecretMetadata},
        fs::MockFileSystem,
        pull::pull_secret_file,
        secret::{MockSecretManagerImpl, Secret, SecretManager},
    };

    #[tokio::test]
    async fn test_pull_secret() {
        let mut mock_secret = MockSecretManagerImpl::new();

        // Expect the "test" secret to be requested
        mock_secret
            .expect_get_secret()
            .times(1)
            .with(eq("test"))
            .return_once(move |_key| Ok(Secret::String("test".to_string())));

        let mut mock_fs = MockFileSystem::new();

        // Expect the ".env" file to be written to
        mock_fs
            .expect_write_file()
            .times(1)
            .with(eq(Path::new("/.env")), eq("test".to_string().into_bytes()))
            .return_once(move |_path, _value| Ok(()));

        let secret = SecretManager::Mock(mock_secret);
        let working_path = Path::new("/");
        let file = SecretFile {
            path: PathBuf::from(".env"),
            secret: "test".to_string(),
            metadata: SecretMetadata::default(),
        };

        pull_secret_file(&mock_fs, &secret, working_path, &file)
            .await
            .unwrap();

        // Unpack the secret manager to perform checkpoint
        let mut mock_secret = match secret {
            SecretManager::Mock(secret) => secret,
            _ => panic!("unexpected secret manager in tests"),
        };

        // Ensure expectations are met
        mock_fs.checkpoint();
        mock_secret.checkpoint();
    }
}
