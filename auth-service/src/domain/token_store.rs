pub enum TokenStoreError {
    UnexpectedError,
}
#[async_trait::async_trait]
pub trait BannableTokenStore: TokenStore + Bannable {}
#[async_trait::async_trait]
pub trait TokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), TokenStoreError>;
    async fn remove_token(&mut self, token: &str) -> Result<(), TokenStoreError>;
}

#[async_trait::async_trait]
pub trait Bannable {
    async fn is_banned(&self, token: &str) -> bool;
}
