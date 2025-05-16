use color_eyre::Report;
use color_eyre::Result;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Unexpected Error")]
    UnexpectedError(#[source] Report),
}
impl PartialEq for BannedTokenStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (
                BannedTokenStoreError::UnexpectedError(_),
                BannedTokenStoreError::UnexpectedError(_)
            )
        )
    }
}
#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError>;
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}
