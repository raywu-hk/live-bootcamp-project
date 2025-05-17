use crate::{Email, Password, User, UserStore, UserStoreError};
use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
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
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let hashed_password = compute_password_hash(user.password.as_ref().to_owned())
            .await
            .map_err(UserStoreError::UnexpectedError)?;
        /*
        let result =
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
            user.email.as_ref().expose_secret(),
            &hashed_password.expose_secret(),
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;
        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserAlreadyExists);
        }
        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
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
        sqlx::query!(
            "select email, password_hash, requires_2fa from users where email = $1",
            email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?
        .map(|user_row| {
            Ok(User {
                email: Email::parse(SecretString::from(user_row.email))
                    .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?,
                password: Password::parse(SecretString::from(user_row.password_hash))
                    .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?,
                requires_2fa: user_row.requires_2fa,
            })
        })
        .ok_or(UserStoreError::UserNotFound)?
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
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
            email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        match user {
            None => Err(UserStoreError::UserNotFound),
            Some(row) => verify_password_hash(
                SecretString::from(row.password_hash),
                password.as_ref().to_owned(),
            )
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

#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: SecretString,
    password_candidate: SecretString,
) -> Result<()> {
    // This line retrieves the current span from the tracing context.
    // The span represents the execution context for the compute_password_hash function.
    let current_span: tracing::Span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        // This code block ensures that the operations within the closure are executed within the context of the current span.
        // This is especially useful for tracing operations that are performed in a different thread or task, such as within tokio::task::spawn_blocking.
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(expected_password_hash.expose_secret())?;
            Argon2::default()
                .verify_password(
                    password_candidate.expose_secret().as_bytes(),
                    &expected_password_hash,
                )
                .map_err(|e| e.into())
        })
    })
    .await?
}

// Helper function to hash passwords before persisting them in the database.
// Hashing is a CPU-intensive operation. To avoid blocking
// other async tasks, update this function to perform hashing on a
// separate thread pool using tokio::task::spawn_blocking. Note that you
// will need to update the input parameters to be String types instead of &str
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: SecretString) -> Result<SecretString> {
    let current_span: tracing::Span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.expose_secret().as_bytes(), &salt)?
            .to_string();
            Ok(SecretString::from(password_hash))
        })
    })
    .await?
}
