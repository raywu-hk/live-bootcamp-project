#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    UnexpectedError,
}
#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError>;
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}
