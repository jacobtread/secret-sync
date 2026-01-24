use crate::{config::SecretFile, fs::FileSystem, secret::SecretManager};
use std::path::Path;

/// Download a secret file from the secret manager
pub async fn pull_secret_file<Fs: FileSystem>(
    fs: &Fs,
    secret: &dyn SecretManager,
    working_path: &Path,
    file: &SecretFile,
) -> eyre::Result<()> {
    let value = secret.get_secret(&file.secret).await?;

    let file_path = if file.path.is_absolute() {
        file.path.clone()
    } else {
        working_path.join(file.path.clone())
    };

    let value: &[u8] = value.as_bytes();
    fs.write_file(&file_path, value).await?;

    Ok(())
}

/// Download a collection of files from the secret manager
pub async fn pull_secret_files<Fs: FileSystem>(
    fs: &Fs,
    secret: &dyn SecretManager,
    working_path: &Path,
    files: impl IntoIterator<Item = &SecretFile>,
) -> eyre::Result<()> {
    for file in files {
        pull_secret_file(fs, secret, working_path, file).await?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        config::{SecretFile, SecretMetadata},
        fs::MockFileSystem,
        pull::{pull_secret_file, pull_secret_files},
        secret::{MockSecretManager, Secret},
    };
    use mockall::{Sequence, predicate::eq};
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
    };

    /// Tests pull a secret file
    #[tokio::test]
    async fn test_pull_secret_file() {
        let mut secret = MockSecretManager::new();

        // Expect the "test" secret to be requested
        secret
            .expect_get_secret()
            .times(1)
            .with(eq("test"))
            .return_once(move |_key| Ok(Secret::String("test".to_string())));

        let mut fs = MockFileSystem::new();

        // Expect the ".env" file to be written to
        fs.expect_write_file()
            .times(1)
            .with(eq(Path::new("/.env")), eq("test".to_string().into_bytes()))
            .return_once(move |_path, _value| Ok(()));

        let working_path = Path::new("/");
        let file = SecretFile {
            path: PathBuf::from(".env"),
            secret: "test".to_string(),
            metadata: SecretMetadata::default(),
        };

        pull_secret_file(&fs, &secret, working_path, &file)
            .await
            .unwrap();

        // Ensure expectations are met
        fs.checkpoint();
        secret.checkpoint();
    }

    /// Tests pulling multiple secret files
    #[tokio::test]
    async fn test_pull_secret_files() {
        const TOTAL_TEST_SECRETS: usize = 50;

        let mut test_secrets: Vec<SecretFile> = Vec::new();
        let mut test_secrets_value: HashMap<String, Secret> = HashMap::new();

        for i in 0..TOTAL_TEST_SECRETS {
            test_secrets.push(SecretFile {
                path: PathBuf::from(format!(".env.{i}")),
                secret: format!("test-{i}"),
                metadata: SecretMetadata::default(),
            });

            test_secrets_value.insert(
                format!("test-{i}"),
                Secret::String(format!("test-{i}-secret")),
            );
        }

        let mut secret = MockSecretManager::new();

        let mut get_secret_sequence = Sequence::new();

        for secret_file in &test_secrets {
            let secret_value = test_secrets_value.get(&secret_file.secret).unwrap().clone();

            // Expect the secret to be requested
            secret
                .expect_get_secret()
                .in_sequence(&mut get_secret_sequence)
                .times(1)
                .with(eq(secret_file.secret.clone()))
                .return_once(move |_key| Ok(secret_value));
        }

        let mut fs = MockFileSystem::new();
        let working_path = Path::new("/");

        let mut write_file_sequence = Sequence::new();
        for secret_file in &test_secrets {
            let secret_value = test_secrets_value.get(&secret_file.secret).unwrap().clone();
            let secret_path = working_path.join(&secret_file.path);

            // Expect the ".env" file to be written to
            fs.expect_write_file()
                .in_sequence(&mut write_file_sequence)
                .times(1)
                .with(eq(secret_path), eq(secret_value.into_bytes()))
                .return_once(move |_path, _value| Ok(()));
        }

        pull_secret_files(&fs, &secret, working_path, &test_secrets)
            .await
            .unwrap();

        // Ensure expectations are met
        fs.checkpoint();
        secret.checkpoint();
    }
}
