use crate::domain::{User, UserStoreError, UserStores};
use std::collections::HashMap;

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}
#[async_trait::async_trait]
impl UserStores for HashmapUserStore {
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
    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        match self.users.get(email) {
            Some(user) => Ok(User::new(
                user.email.to_string(),
                user.password.to_string(),
                false,
            )),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    // Implement a public method called `validate_user`, which takes an
    // immutable reference to self, an email string slice, and a password string slice
    // as arguments. `validate_user` should return a `Result` type containing either a
    // unit type `()` if the email/password passed in match an existing user, or a `UserStoreError`.
    // Return `UserStoreError::UserNotFound` if the user can not be found.
    // Return `UserStoreError::InvalidCredentials` if the password is incorrect.
    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
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
    use crate::domain::{User, UserStores};
    use crate::services::hashmap_user_store::{HashmapUserStore, UserStoreError};

    #[tokio::test]
    async fn test_add_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com".to_string(), "password".to_string(), false);

        let result = user_store.add_user(user).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com".to_string(), "password".to_string(), false);
        let result = user_store.add_user(user).await;
        assert!(result.is_ok());

        let expected_user = User::new("test@test.com".to_string(), "password".to_string(), false);
        let result_user = user_store.get_user("test@test.com").await.unwrap();

        assert_eq!(result_user, expected_user);

        let result = user_store.get_user("notExist@notExist.com").await;

        assert_eq!(result, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut user_store = HashmapUserStore::default();
        let user = User::new("test@test.com".to_string(), "password".to_string(), false);
        let result = user_store.add_user(user).await;
        assert!(result.is_ok());

        let result = user_store.validate_user("test@test.com", "password").await;
        assert!(result.is_ok());

        let result = user_store
            .validate_user("test@test.com", "wrong_password")
            .await;
        assert_eq!(result, Err(UserStoreError::InvalidCredentials));
    }
}
