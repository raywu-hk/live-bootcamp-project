use crate::domain::{BannedTokenStore, BannedTokenStoreError};
use secrecy::{ExposeSecret, SecretString};
use std::collections::HashSet;

#[derive(Default)]
pub struct HashSetBannedTokenStore {
    tokens: HashSet<String>,
}
#[async_trait::async_trait]
impl BannedTokenStore for HashSetBannedTokenStore {
    async fn add_token(&mut self, token: &SecretString) -> Result<(), BannedTokenStoreError> {
        self.tokens.insert(token.expose_secret().to_owned());
        Ok(())
    }
    async fn contains_token(&self, token: &SecretString) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.contains(token.expose_secret()))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::BannedTokenStore;
    use crate::services::hashset_banned_token_store::HashSetBannedTokenStore;
    use secrecy::SecretString;

    #[tokio::test]
    async fn test_is_banned() {
        let mut token_store = HashSetBannedTokenStore::default();
        let token = SecretString::from("token");
        let result = token_store.add_token(&token).await;
        assert!(result.is_ok());

        let result = token_store.contains_token(&token).await;
        assert_eq!(result, Ok(true));
    }
    #[tokio::test]
    async fn test_add_token() {
        let mut token_store = HashSetBannedTokenStore::default();
        let token = SecretString::from("token");

        let result = token_store.add_token(&token).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_add_token_already_exists() {
        let mut token_store = HashSetBannedTokenStore::default();
        let token = SecretString::from("token");

        let _result = token_store.add_token(&token).await;
        let result = token_store.add_token(&token).await;
        assert!(result.is_ok());
    }
}
