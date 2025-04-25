use crate::domain::{Bannable, BannableTokenStore, TokenStore, TokenStoreError};
use std::collections::HashSet;
#[derive(Default)]
pub struct HashMapBannedTokenStore {
    tokens: HashSet<String>,
}
#[async_trait::async_trait]
impl BannableTokenStore for HashMapBannedTokenStore {}
#[async_trait::async_trait]
impl TokenStore for HashMapBannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), TokenStoreError> {
        self.tokens.insert(token.to_string());
        Ok(())
    }

    async fn remove_token(&mut self, token: &str) -> Result<(), TokenStoreError> {
        if self.tokens.remove(token) {
            Ok(())
        } else {
            Err(TokenStoreError::UnexpectedError)
        }
    }
}

#[async_trait::async_trait]
impl Bannable for HashMapBannedTokenStore {
    async fn is_banned(&self, token: &str) -> bool {
        self.tokens.contains(token)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{Bannable, TokenStore};
    use crate::services::hashmap_banned_token_store::HashMapBannedTokenStore;

    #[tokio::test]
    async fn test_is_banned() {
        let mut token_store = HashMapBannedTokenStore::default();

        let result = token_store.add_token("token").await;
        assert!(result.is_ok());

        let result = token_store.is_banned("token").await;
        assert!(result);
    }
    #[tokio::test]
    async fn test_add_token() {
        let mut token_store = HashMapBannedTokenStore::default();

        let result = token_store.add_token("token").await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_add_token_already_exists() {
        let mut token_store = HashMapBannedTokenStore::default();

        let _result = token_store.add_token("token").await;
        let result = token_store.add_token("token").await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_remove_token() {
        let mut token_store = HashMapBannedTokenStore::default();

        let result = token_store.add_token("token").await;
        assert!(result.is_ok());

        let result = token_store.remove_token("token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_token_not_exist() {
        let mut token_store = HashMapBannedTokenStore::default();

        let result = token_store.remove_token("nonexistent").await;

        assert!(result.is_err());
    }
}
