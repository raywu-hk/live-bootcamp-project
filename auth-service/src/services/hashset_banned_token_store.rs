use crate::domain::{BannedTokenStore, BannedTokenStoreError};
use std::collections::HashSet;

#[derive(Default)]
pub struct HashSetBannedTokenStore {
    tokens: HashSet<String>,
}
#[async_trait::async_trait]
impl BannedTokenStore for HashSetBannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        self.tokens.insert(token.to_string());
        Ok(())
    }
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::BannedTokenStore;
    use crate::services::hashset_banned_token_store::HashSetBannedTokenStore;

    #[tokio::test]
    async fn test_is_banned() {
        let mut token_store = HashSetBannedTokenStore::default();

        let result = token_store.add_token("token").await;
        assert!(result.is_ok());

        let result = token_store.contains_token("token").await;
        assert_eq!(result, Ok(true));
    }
    #[tokio::test]
    async fn test_add_token() {
        let mut token_store = HashSetBannedTokenStore::default();

        let result = token_store.add_token("token").await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_add_token_already_exists() {
        let mut token_store = HashSetBannedTokenStore::default();

        let _result = token_store.add_token("token").await;
        let result = token_store.add_token("token").await;
        assert!(result.is_ok());
    }
}
