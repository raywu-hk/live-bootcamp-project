use crate::utils::TOKEN_TTL_SECONDS;
use crate::{BannedTokenStore, BannedTokenStoreError};
use color_eyre::eyre::Context;
use redis::{Commands, Connection};
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "Adding token in Redis", skip_all)]
    async fn add_token(&mut self, token: &SecretString) -> Result<(), BannedTokenStoreError> {
        // 1. Create a new key using the get_key helper function.
        // 2. Call the set_ex command on the Redis connection to set a new key/value pair with an expiration time (TTL).
        // The value should simply be a `true` (boolean value).
        // The expiration time should be set to TOKEN_TTL_SECONDS.
        // NOTE: The TTL is expected to be a u64 so you will have to cast TOKEN_TTL_SECONDS to a u64.
        // Return BannedTokenStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        let ttl: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .wrap_err("failed to cast TOKEN_TTL_SECONDS to u64")
            .map_err(BannedTokenStoreError::UnexpectedError)?;
        let key = get_key(token.expose_secret());
        let _: () = self
            .conn
            .write()
            .await
            .set_ex(&key, true, ttl)
            .wrap_err("failed to set banned token in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;
        Ok(())
    }

    #[tracing::instrument(name = "Contains token", skip_all)]
    async fn contains_token(&self, token: &SecretString) -> Result<bool, BannedTokenStoreError> {
        // Check if the token exists by calling the exists method on the Redis connection
        let key = get_key(token.expose_secret());
        let is_banned = self
            .conn
            .write()
            .await
            .exists(key)
            .wrap_err("fail to check if token exists in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;
        Ok(is_banned)
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
