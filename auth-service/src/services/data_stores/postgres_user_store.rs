use crate::{Email, Password, User, UserStore, UserStoreError};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use sqlx::PgPool;
use std::error::Error;

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    // Implement all required methods. Note that you will need to make SQL queries against our PostgreSQL instance inside these methods.

    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let hashed_password = compute_password_hash(user.password.as_ref().to_string())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;
        /*        let result =
        sqlx::query("insert into users (email,password_hash,requires_2fa) values ($1,$2,$3)")
            .bind(user.email)
            .bind(hashed_password)
            .bind(user.requires_2fa)
            .execute(&self.pool)
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;
            */
        let result = sqlx::query!(
            "insert into users (email,password_hash,requires_2fa) values ($1,$2,$3)",
            user.email.as_ref(),
            hashed_password,
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;
        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserAlreadyExists);
        }
        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        /*
        let user = sqlx::query_as::<_, User>(
            "select
                email,
                password_hash as password,
                requires_2fa from users where email = $1",
        )
        .bind(email.as_ref())
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;
        */
        let user = sqlx::query_as!(
            User,
            "select users.email, users.password_hash as password, users.requires_2fa from users where email = $1",
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;
        match user {
            None => Err(UserStoreError::UserNotFound),
            Some(user) => Ok(user),
        }
    }

    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        /*
        let user = sqlx::query("select email,password_hash from users where email = $1")
                    .bind(email.as_ref())
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|_| UserStoreError::UnexpectedError)?;
                    */
        let user = sqlx::query!(
            "select users.email,users.password_hash from users where email = $1",
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        match user {
            None => Err(UserStoreError::UserNotFound),
            Some(row) => verify_password_hash(row.password_hash, password.as_ref().to_string())
                .await
                .map_err(|_| UserStoreError::InvalidCredentials),
        }
    }
}

// Helper function to verify if a given password matches an expected hash
// Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;

        Argon2::default()
            .verify_password(password_candidate.as_bytes(), &expected_password_hash)
            .map_err(|e| e.into())
    })
    .await?
}

// Helper function to hash passwords before persisting them in the database.
// Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        )
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
        Ok(password_hash)
    })
    .await?
}
