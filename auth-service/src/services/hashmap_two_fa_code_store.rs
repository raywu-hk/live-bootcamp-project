use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use std::collections::HashMap;

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

// implement TwoFACodeStore for HashmapTwoFACodeStore
#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email, (login_attempt_id, code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        if self.codes.remove(email).is_none() {
            Err(TwoFACodeStoreError::UnexpectedError(eyre!(
                "Email not found in HashmapTwoFACodeStore"
            )))?
        }
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some(v) => Ok(v.clone()),
            None => Err(TwoFACodeStoreError::UnexpectedError(eyre!(
                "Email not found in HashmapTwoFACodeStore"
            ))),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_add_code_successfully() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@example.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        let result = store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_remove_code_successfully() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@example.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store
            .add_code(email.clone(), login_attempt_id, code)
            .await
            .unwrap();
        let result = store.remove_code(&email).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_get_code_successfully() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@example.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await
            .unwrap();
        let result = store.get_code(&email).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), (login_attempt_id, code));
    }

    #[tokio::test]
    async fn should_return_error_when_getting_non_existent_code() {
        let store = HashmapTwoFACodeStore::default();
        let email = Email::parse("test@example.com").unwrap();

        let result = store.get_code(&email).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            TwoFACodeStoreError::UnexpectedError(eyre!("Email not found in HashmapTwoFACodeStore"))
        );
    }
}
