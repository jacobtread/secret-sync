use crate::config::SecretMetadata;
use async_trait::async_trait;
use mockall::automock;

pub mod aws;

pub enum Secret {
    String(String),
    Binary(Vec<u8>),
}

#[automock]
#[async_trait]
pub trait SecretManager {
    async fn get_secret(&self, name: &str) -> eyre::Result<Secret>;

    async fn set_secret(
        &self,
        name: &str,
        value: Secret,
        metadata: &SecretMetadata,
    ) -> eyre::Result<()>;
}
