//! # AWS
//!
//! Secret manager backed by AWS secret manager and compatible secret
//! managers

use super::Secret;
use crate::{
    config::{AwsConfig, SecretMetadata},
    secret::SecretManager,
};
use async_trait::async_trait;
use aws_config::{
    BehaviorVersion, Region,
    meta::region::{ProvideRegion, RegionProviderChain},
};
use aws_sdk_secretsmanager::{
    config::{Credentials, SharedCredentialsProvider},
    primitives::Blob,
    types::Tag,
};
use eyre::Context;

pub struct AwsSecretManager {
    client: aws_sdk_secretsmanager::Client,
}

impl AwsSecretManager {
    /// Create a [AwsSecretManager] from the provided `config`
    pub async fn from_config(config: &AwsConfig) -> eyre::Result<AwsSecretManager> {
        // Setup the region provider
        let region_provider: Box<dyn ProvideRegion> = match config.region.as_ref() {
            Some(value) => Box::new(Region::new(value.clone())),
            None => Box::new(RegionProviderChain::default_provider().or_else("us-east-1")),
        };

        // Load the base configuration from env variables
        // (See https://docs.aws.amazon.com/sdkref/latest/guide/settings-reference.html#EVarSettings)
        let mut builder = aws_config::from_env()
            .region(region_provider)
            .behavior_version(BehaviorVersion::v2026_01_12());

        if let Some(profile) = config.profile.as_ref() {
            builder = builder.profile_name(profile);
        }

        if let Some(endpoint) = config.endpoint.as_ref() {
            builder = builder.endpoint_url(endpoint);
        }

        if let Some(credentials) = config.credentials.as_ref() {
            let credentials = Credentials::new(
                credentials.access_key_id.clone(),
                credentials.access_key_secret.clone(),
                None,
                None,
                "secret_sync",
            );

            builder = builder.credentials_provider(SharedCredentialsProvider::new(credentials));
        }

        let sdk_config = builder.load().await;

        let client = aws_sdk_secretsmanager::Client::new(&sdk_config);

        Ok(Self { client })
    }
}

#[async_trait]
impl SecretManager for AwsSecretManager {
    async fn get_secret(&self, name: &str) -> eyre::Result<Secret> {
        let result = match self.client.get_secret_value().secret_id(name).send().await {
            Ok(value) => value,
            Err(error) => {
                if error
                    .as_service_error()
                    .is_some_and(|value| value.is_resource_not_found_exception())
                {
                    eyre::bail!("secret \"{name}\" not found")
                }

                tracing::error!(?error, "failed to get secret value");
                return Err(eyre::Report::new(error));
            }
        };

        if let Some(value) = result.secret_string {
            return Ok(Secret::String(value));
        }

        if let Some(value) = result.secret_binary {
            return Ok(Secret::Binary(value.into_inner()));
        }

        eyre::bail!("no valid secret found for \"{name}\" ")
    }

    async fn set_secret(
        &self,
        name: &str,
        value: Secret,
        metadata: &SecretMetadata,
    ) -> eyre::Result<()> {
        let (secret_binary, secret_string) = match value {
            Secret::String(value) => (None, Some(value)),
            Secret::Binary(items) => (Some(Blob::new(items)), None),
        };

        let tags = metadata.tags.as_ref().map(|tags| {
            tags.iter()
                .map(|(key, value)| Tag::builder().key(key).value(value).build())
                .collect::<Vec<_>>()
        });

        let error = match self
            .client
            .create_secret()
            .set_secret_binary(secret_binary.clone())
            .set_secret_string(secret_string.clone())
            .set_description(metadata.description.clone())
            .set_tags(tags)
            .name(name)
            .send()
            .await
        {
            Ok(_) => return Ok(()),
            Err(err) => err,
        };

        // Handle secret already existing
        if error
            .as_service_error()
            .is_some_and(|value| value.is_resource_exists_exception())
        {
            tracing::debug!("secret already exists, updating secret");

            self.client
                .update_secret()
                .set_secret_binary(secret_binary)
                .set_secret_string(secret_string)
                .secret_id(name)
                .send()
                .await
                .inspect_err(|error| {
                    tracing::error!(?error, "failed to update secret");
                })
                .context("failed to update secret")?;

            return Ok(());
        }

        tracing::error!(?error, "failed to create secret");
        Err(eyre::Report::new(error))
    }
}
