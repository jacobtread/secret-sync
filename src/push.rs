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
    files: impl IntoIterator<Item = &SecretFile>,
) -> eyre::Result<()> {
    for file in files {
        push_secret_file(fs, secret, working_path, file).await?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        config::{SecretFile, SecretMetadata},
        fs::MockFileSystem,
        push::{push_secret_file, push_secret_files},
        secret::{MockSecretManager, Secret},
    };
    use mockall::{Sequence, predicate::eq};
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
    };

    /// Tests pushing a secret file
    #[tokio::test]
    async fn test_push_secret_file() {
        let mut secret = MockSecretManager::new();

        // Expect the "test" secret to be set
        secret
            .expect_set_secret()
            .times(1)
            .with(
                eq("test"),
                eq(Secret::String("test".to_string())),
                eq(SecretMetadata::default()),
            )
            .return_once(move |_key, _secret, _metadata| Ok(()));

        let mut fs = MockFileSystem::new();

        // Expect the ".env" file to be written to
        fs.expect_read_file()
            .times(1)
            .with(eq(Path::new("/.env")))
            .return_once(move |_path| Ok("test".to_string().into_bytes()));

        let working_path = Path::new("/");
        let file = SecretFile {
            path: PathBuf::from(".env"),
            secret: "test".to_string(),
            metadata: SecretMetadata::default(),
        };

        push_secret_file(&fs, &secret, working_path, &file)
            .await
            .unwrap();

        // Ensure expectations are met
        fs.checkpoint();
        secret.checkpoint();
    }

    /// Tests pushing multiple secret files
    #[tokio::test]
    async fn test_push_secret_files() {
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

        let mut set_secret_sequence = Sequence::new();

        for secret_file in &test_secrets {
            let secret_value = test_secrets_value.get(&secret_file.secret).unwrap().clone();

            // Expect the secret to be set
            // Expect the "test" secret to be requested
            secret
                .expect_set_secret()
                .in_sequence(&mut set_secret_sequence)
                .times(1)
                .with(
                    eq(secret_file.secret.clone()),
                    eq(secret_value),
                    eq(secret_file.metadata.clone()),
                )
                .return_once(move |_key, _secret, _metadata| Ok(()));
        }

        let mut fs = MockFileSystem::new();
        let working_path = Path::new("/");

        let mut read_file_sequence = Sequence::new();
        for secret_file in &test_secrets {
            let secret_value = test_secrets_value.get(&secret_file.secret).unwrap().clone();
            let secret_path = working_path.join(&secret_file.path);

            // Expect the ".env" file to be read from
            fs.expect_read_file()
                .in_sequence(&mut read_file_sequence)
                .times(1)
                .with(eq(secret_path))
                .return_once(move |_path| Ok(secret_value.into_bytes()));
        }

        push_secret_files(&fs, &secret, working_path, &test_secrets)
            .await
            .unwrap();

        // Ensure expectations are met
        fs.checkpoint();
        secret.checkpoint();
    }
}
