use crate::domain::Email;
use rand::Rng;
use std::str::from_utf8;
use uuid::Uuid;

// This trait represents the interface all concrete 2FA code stores should implement
#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        // Use the `parse_str` function from the `uuid` crate to ensure `id` is a valid UUID
        match Uuid::parse_str(&id) {
            Ok(uuid) => Ok(Self(uuid.to_string())),
            Err(_) => Err(format!("{} is not a valid uuid", id)),
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        // Use the `uuid` crate to generate a random version 7 UUID
        LoginAttemptId(Uuid::now_v7().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        // Ensure `code` is a valid 6-digit code
        match code.len() == 6 {
            true => Ok(Self(code)),
            false => Err(format!("two-fa-code {} is not 6 digit", code)),
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        // Use the `rand` crate to generate a random 2FA code.
        // The code should be 6 digits (ex: 834629)
        let mut arr = [0u8; 6];
        rand::rng().fill(&mut arr[..]);
        let code = from_utf8(&arr).unwrap().to_string();
        TwoFACode(code)
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
