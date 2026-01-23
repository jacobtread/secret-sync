use crate::config::SecretMetadata;
use async_trait::async_trait;
use mockall::automock;

pub mod aws;

#[derive(Clone, PartialEq)]
pub enum Secret {
    String(String),
    Binary(Vec<u8>),
}

impl Secret {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Secret::String(value) => value.as_bytes(),
            Secret::Binary(value) => value,
        }
    }

    #[cfg(test)]
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Secret::String(value) => value.clone().into_bytes(),
            Secret::Binary(value) => value.clone(),
        }
    }
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
