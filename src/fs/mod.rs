use mockall::automock;
use std::path::Path;

pub mod real;

/// File system abstraction
#[automock]
pub trait FileSystem {
    /// Read a file from the provided `path`
    async fn read_file(&self, path: &Path) -> eyre::Result<Vec<u8>>;

    /// Write the provided `bytes` to the file at `path`
    async fn write_file(&self, path: &Path, bytes: &[u8]) -> eyre::Result<()>;
}
