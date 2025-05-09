use std::sync::Arc;

use crate::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};
use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        // 1. Create a new key using the get_key helper function.
        // 2. Create a TwoFATuple instance.
        // 3. Use serde_json::to_string to serialize the TwoFATuple instance into a JSON string.
        // Return TwoFACodeStoreError::UnexpectedError if serialization fails.
        // 4. Call the set_ex command on the Redis connection to set a new key/value pair with an expiration time (TTL).
        // The value should be the serialized 2FA tuple.
        // The expiration time should be set to TEN_MINUTES_IN_SECONDS.
        // Return TwoFACodeStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        let key = get_key(&email);
        let two_fa_tuple = (login_attempt_id, code);
        let json_str = serde_json::to_string(&two_fa_tuple)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        self.conn
            .write()
            .await
            .set_ex(key, json_str, TEN_MINUTES_IN_SECONDS)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        // 1. Create a new key using the get_key helper function.
        // 2. Call the del command on the Redis connection to delete the 2FA code entry.
        // Return TwoFACodeStoreError::UnexpectedError if the operation fails.
        let key = get_key(email);
        self.conn
            .write()
            .await
            .del(key)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        // 1. Create a new key using the get_key helper function.
        // 2. Call the get command on the Redis connection to get the value stored for the key.
        // Return TwoFACodeStoreError::LoginAttemptIdNotFound if the operation fails.
        // If the operation succeeds, call serde_json::from_str to parse the JSON string into a TwoFATuple.
        // Then, parse the login attempt ID string and 2FA code string into a LoginAttemptId and TwoFACode type respectively.
        // Return TwoFACodeStoreError::UnexpectedError if parsing fails.
        let key = get_key(email);
        let two_fa_tuple: String = self
            .conn
            .write()
            .await
            .get(key)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        serde_json::from_str::<(LoginAttemptId, TwoFACode)>(&two_fa_tuple)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
