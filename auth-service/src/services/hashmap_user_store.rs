use crate::domain::{Email, Password, User, UserStore, UserStoreError};
use std::collections::HashMap;

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}
#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        // Return `UserStoreError::UserAlreadyExists` if the user already exists,
        // otherwise insert the user into the hashmap and return `Ok(())`.
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        };

        self.users.insert(user.email.clone(), user);

        Ok(())
    }

    // Implement a public method called `get_user`, which takes an
    // immutable reference to self and an email string slice as arguments.
    // This function should return a `Result` type containing either a
    // `User` object or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(user.clone()),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    // Implement a public method called `validate_user`, which takes an
    // immutable reference to self, an email string slice, and a password string slice
    // as arguments. `validate_user` should return a `Result` type containing either a
    // unit type `()` if the email/password passed in match an existing user, or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    // Return `UserStoreError::InvalidCredentials` if the password is incorrect.
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        let user = self.users.get(email).ok_or(UserStoreError::UserNotFound)?;
        if user.password.ne(password) {
            return Err(UserStoreError::InvalidCredentials);
        }
        Ok(())
    }
}

// Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use crate::domain::{Email, Password, User, UserStore};
    use crate::services::hashmap_user_store::{HashmapUserStore, UserStoreError};

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@test.com").unwrap();
        let password = Password::parse("password").unwrap();
        let user = User::new(email.clone(), password.clone(), false);

        let result = user_store.add_user(user).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@test.com").unwrap();
        let password = Password::parse("password").unwrap();
        let user = User::new(email.clone(), password.clone(), false);
        let result = user_store.add_user(user).await;
        assert!(result.is_ok());

        let result = user_store
            .get_user(&Email::parse("notExist@notExist.com").unwrap())
            .await;
        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut user_store = HashmapUserStore::default();
        let email = Email::parse("test@test.com").unwrap();
        let password = Password::parse("password").unwrap();
        let user = User::new(email.clone(), password.clone(), false);
        let result = user_store.add_user(user).await;
        assert!(result.is_ok());

        let result = user_store
            .validate_user(
                &Email::parse("test@test.com").unwrap(),
                &Password::parse("password").unwrap(),
            )
            .await;
        assert!(result.is_ok());

        let result = user_store
            .validate_user(
                &Email::parse("test@test.com").unwrap(),
                &Password::parse("wrong_password").unwrap(),
            )
            .await;
        assert_eq!(result, Err(UserStoreError::InvalidCredentials));
    }
}
