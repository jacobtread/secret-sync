//! # Secret Manager
//!
//! This module contains a secret manager implementation generalizing the behavior
//! a secret manager must have so that it is abstracted for pulling and pushing.
//!
//! - [`aws`] AWS Compatible secret manager backend

use crate::config::SecretMetadata;
use async_trait::async_trait;
use mockall::automock;
use std::fmt::Debug;

pub mod aws;

/// Secret value
#[derive(Clone, PartialEq, Eq)]
pub enum Secret {
    /// UTF-8 encoded secret
    String(String),
    /// Generic binary secret
    Binary(Vec<u8>),
}

impl Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Secret").field(&"< REDACTED >").finish()
    }
}

impl Secret {
    /// Get the secret as a slice of bytes
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Secret::String(value) => value.as_bytes(),
            Secret::Binary(value) => value,
        }
    }

    /// Convert the secret into bytes
    #[cfg(test)]
    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Secret::String(value) => value.into_bytes(),
            Secret::Binary(value) => value,
        }
    }
}

/// Secret manager abstraction
#[automock]
#[async_trait]
pub trait SecretManager {
    /// Get a secret from the secret manager by `name`
    async fn get_secret(&self, name: &str) -> eyre::Result<Secret>;

    /// Set a secret by `name` to `value` with some `metadata`
    async fn set_secret(
        &self,
        name: &str,
        value: Secret,
        metadata: &SecretMetadata,
    ) -> eyre::Result<()>;
}
