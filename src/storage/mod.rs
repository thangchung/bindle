pub mod file;

#[cfg(test)]
pub(crate) mod test_common;

use std::convert::TryInto;

use thiserror::Error;
use tokio::io::AsyncRead;

use crate::id::ParseError;
use crate::Id;

pub type Result<T> = core::result::Result<T, StorageError>;

#[async_trait::async_trait]
pub trait Storage {
    /// This takes an invoice and creates it in storage.
    /// It must verify that each referenced box is present in storage. Any box that
    /// is not present must be returned in the list of IDs.
    async fn create_invoice(&self, inv: &super::Invoice) -> Result<Vec<super::Label>>;
    // Load an invoice and return it
    //
    // This will return an invoice if the bindle exists and is not yanked
    async fn get_invoice<I>(&self, id: I) -> Result<super::Invoice>
    where
        I: TryInto<Id, Error = ParseError> + Send;
    // Load an invoice, even if it is yanked.
    async fn get_yanked_invoice<I>(&self, id: I) -> Result<super::Invoice>
    where
        I: TryInto<Id, Error = ParseError> + Send;
    // Remove an invoice by ID
    async fn yank_invoice<I>(&self, id: I) -> Result<()>
    where
        I: TryInto<Id, Error = ParseError> + Send;
    async fn create_parcel<R: AsyncRead + Unpin + Send + Sync>(
        &self,
        label: &super::Label,
        data: &mut R,
    ) -> Result<()>;

    async fn get_parcel(&self, parcel_id: &str) -> Result<Box<dyn AsyncRead + Unpin + Send>>;
    // Get the label for a parcel
    //
    // This reads the label from storage and then parses it into a Label object.
    async fn get_label(&self, parcel_id: &str) -> Result<crate::Label>;
}

/// StorageError describes the possible error states when storing and retrieving bindles.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("bindle is yanked")]
    Yanked,
    #[error("bindle cannot be created as yanked")]
    CreateYanked,
    #[error("resource not found")]
    NotFound,
    #[error("resource could not be loaded")]
    IO(#[from] std::io::Error),
    #[error("resource already exists")]
    Exists,
    #[error("Invalid ID given")]
    InvalidId,
    #[error("digest does not match")]
    DigestMismatch,

    // TODO: Investigate how to make this more helpful
    #[error("resource is malformed")]
    Malformed(#[from] toml::de::Error),
    #[error("resource cannot be stored")]
    Unserializable(#[from] toml::ser::Error),
}

impl From<crate::id::ParseError> for StorageError {
    fn from(e: crate::id::ParseError) -> StorageError {
        match e {
            crate::id::ParseError::InvalidId => StorageError::InvalidId,
        }
    }
}
