//! # Real
//!
//! File system backed by the real host file system

use crate::fs::FileSystem;
use eyre::{Context, ContextCompat};
use tokio::fs::create_dir_all;

/// File system backed by real files
pub struct RealFs;

impl FileSystem for RealFs {
    #[tracing::instrument(skip(self))]
    async fn read_file(&self, path: &std::path::Path) -> eyre::Result<Vec<u8>> {
        if !path.exists() {
            eyre::bail!("cannot push secret, file does not exist");
        }

        let value = tokio::fs::read(&path)
            .await
            .context("failed to read secret file")?;

        Ok(value)
    }

    #[tracing::instrument(skip(self, bytes))]
    async fn write_file(&self, path: &std::path::Path, bytes: &[u8]) -> eyre::Result<()> {
        let parent_path = path.parent().context("file parent path does not exist")?;

        if !parent_path.exists() {
            tracing::debug!(
                ?path,
                ?parent_path,
                "path does not exist, creating parent path"
            );

            create_dir_all(parent_path)
                .await
                .context("failed to create parent directory for secret file")?;
        }

        tokio::fs::write(path, bytes)
            .await
            .context("failed to write secret to file")?;

        Ok(())
    }
}
