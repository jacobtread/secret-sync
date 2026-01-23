use mockall::automock;

use crate::{
    config::{BackendProvider, Config, SecretMetadata},
    secret::aws::AwsSecretManager,
};

mod aws;

pub enum SecretManager {
    Aws(AwsSecretManager),
    #[cfg(test)]
    Mock(MockSecretManagerImpl),
}

impl SecretManager {
    pub async fn from_config(config: &Config) -> eyre::Result<SecretManager> {
        match config.backend.provider {
            BackendProvider::Aws => AwsSecretManager::from_config(&config.aws)
                .await
                .map(SecretManager::Aws),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_secret(&self, name: &str) -> eyre::Result<Secret> {
        match self {
            SecretManager::Aws(secret) => secret.get_secret(name).await,
            #[cfg(test)]
            SecretManager::Mock(secret) => secret.get_secret(name).await,
        }
    }

    #[tracing::instrument(skip(self, value))]
    pub async fn set_secret(
        &self,
        name: &str,
        value: Secret,
        metadata: &SecretMetadata,
    ) -> eyre::Result<()> {
        match self {
            SecretManager::Aws(secret) => secret.set_secret(name, value, metadata).await,
            #[cfg(test)]
            SecretManager::Mock(secret) => secret.set_secret(name, value, metadata).await,
        }
    }
}

pub enum Secret {
    String(String),
    Binary(Vec<u8>),
}

#[automock]
pub(crate) trait SecretManagerImpl {
    async fn get_secret(&self, name: &str) -> eyre::Result<Secret>;

    async fn set_secret(
        &self,
        name: &str,
        value: Secret,
        metadata: &SecretMetadata,
    ) -> eyre::Result<()>;
}
