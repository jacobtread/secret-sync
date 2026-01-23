use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_secretsmanager::config::{Credentials, SharedCredentialsProvider};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use toml::Table;

const TEST_ENCRYPTION_KEY: &str = "test";
const TEST_ACCESS_KEY_ID: &str = "test";
const TEST_ACCESS_KEY_SECRET: &str = "test";

/// Create a new [Loker](https://github.com/jacobtread/loker) container for testing
#[allow(dead_code)]
pub async fn test_loker_container() -> ContainerAsync<GenericImage> {
    GenericImage::new("jacobtread/loker", "0.2.2")
        .with_exposed_port(8080.tcp())
        .with_wait_for(WaitFor::seconds(5))
        .with_env_var("SM_ENCRYPTION_KEY", TEST_ENCRYPTION_KEY)
        .with_env_var("SM_ACCESS_KEY_ID", TEST_ACCESS_KEY_ID)
        .with_env_var("SM_ACCESS_KEY_SECRET", TEST_ACCESS_KEY_SECRET)
        .start()
        .await
        .unwrap()
}

/// Create an AWS sdk config for use in tests
#[allow(dead_code)]
pub async fn test_container_sdk_config(container: &ContainerAsync<GenericImage>) -> SdkConfig {
    let host = container.get_host().await.unwrap();
    let host_port = container.get_host_port_ipv4(8080).await.unwrap();
    let url = format!("http://{host}:{host_port}");
    test_sdk_config(
        &url,
        Credentials::new(
            TEST_ACCESS_KEY_ID,
            TEST_ACCESS_KEY_SECRET,
            None,
            None,
            "tests",
        ),
    )
}

/// Create an AWS sdk config for use in tests
#[allow(dead_code)]
pub async fn test_container_secret_client(
    container: &ContainerAsync<GenericImage>,
) -> aws_sdk_secretsmanager::Client {
    let config = test_container_sdk_config(container).await;
    aws_sdk_secretsmanager::Client::new(&config)
}

#[allow(dead_code)]
pub fn test_sdk_config(endpoint_url: &str, credentials: Credentials) -> SdkConfig {
    SdkConfig::builder()
        .behavior_version(BehaviorVersion::v2026_01_12())
        .region(Region::from_static("us-east-1"))
        .endpoint_url(endpoint_url)
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .build()
}

/// Create a new AWS secrets manager test config from the provided Loker container
#[allow(dead_code)]
pub async fn test_config_base(container: &ContainerAsync<GenericImage>) -> Table {
    let host = container.get_host().await.unwrap();
    let host_port = container.get_host_port_ipv4(8080).await.unwrap();
    let url = format!("http://{host}:{host_port}");

    toml::toml! {
        [aws]
        endpoint = url

        [aws.credentials]
        access_key_id = TEST_ACCESS_KEY_ID
        access_key_secret = TEST_ACCESS_KEY_SECRET
    }
}
