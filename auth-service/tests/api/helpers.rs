use auth_service::utils::test::APP_ADDRESS;
use auth_service::utils::{DATABASE_URL, REDIS_HOST_NAME, test};
use auth_service::{
    AppState, Application, BannedStoreType, Email, PostgresUserStore, PostmarkEmailClient,
    RedisBannedTokenStore, RedisTwoFACodeStore, TwoFACodeStoreType, get_postgres_pool,
    get_redis_client,
};
use reqwest::Client;
use reqwest::cookie::Jar;
use secrecy::{ExposeSecret, SecretString};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use wiremock::MockServer;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub banned_token_store: BannedStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub http_client: Client,
    pub email_server: MockServer,
    pub db_name: String,
    pub clean_up_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let (db_name, pg_pool) = Self::configure_postgresql().await;
        let redis_pool = Arc::new(RwLock::new(Self::configure_redis().await));
        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store =
            Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_pool.clone())));
        let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_pool.clone())));
        let email_server = MockServer::start().await;
        let base_url = email_server.uri();
        let email_client = Arc::new(Self::configure_postmark_email_client(base_url));
        // let email_client = Arc::new(MockEmailClient {});
        let app_state = AppState::new(
            user_store.clone(),
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
        );
        let app = Application::build(app_state, APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap(); // Create a Reqwest http client instance

        // Create a new ` TestApp ` instance and return it
        Self {
            address,
            cookie_jar,
            banned_token_store,
            two_fa_code_store,
            http_client,
            email_server,
            db_name,
            clean_up_called: false,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    async fn configure_postgresql() -> (String, PgPool) {
        let postgresql_conn_url = DATABASE_URL.to_owned();

        // We are creating a new database for each test case, and we need to ensure each database has a unique name!
        let db_name = Uuid::now_v7().to_string();

        Self::configure_database(&postgresql_conn_url.expose_secret(), &db_name).await;

        let postgresql_conn_url_with_db =
            format!("{}/{}", postgresql_conn_url.expose_secret(), db_name);

        // Create a new connection pool and return it
        let pg_pool = get_postgres_pool(&SecretString::from(postgresql_conn_url_with_db))
            .await
            .expect("Failed to create Postgres connection pool!");
        (db_name, pg_pool)
    }

    async fn configure_database(db_conn_string: &str, db_name: &str) {
        // Create a database connection
        let connection = PgPoolOptions::new()
            .connect(db_conn_string)
            .await
            .expect("Failed to create Postgres connection pool.");

        // Create a new database
        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
            .await
            .expect("Failed to create database.");

        // Connect to new database
        let db_conn_string = format!("{}/{}", db_conn_string, db_name);

        let connection = PgPoolOptions::new()
            .connect(&db_conn_string)
            .await
            .expect("Failed to create Postgres connection pool.");

        // Run migrations against new database
        sqlx::migrate!()
            .run(&connection)
            .await
            .expect("Failed to migrate the database");
    }

    async fn configure_redis() -> redis::Connection {
        get_redis_client(REDIS_HOST_NAME.to_owned())
            .expect("Failed to get Redis client")
            .get_connection()
            .expect("Failed to get Redis connection")
    }

    fn configure_postmark_email_client(base_url: String) -> PostmarkEmailClient {
        let postmark_auth_token = SecretString::from("auth_token");

        let sender = Email::parse(SecretString::from(test::email_client::SENDER)).unwrap();

        let http_client = Client::builder()
            .timeout(test::email_client::TIMEOUT)
            .build()
            .expect("Failed to build HTTP client");

        PostmarkEmailClient::new(base_url, sender, postmark_auth_token, http_client)
    }

    async fn delete_database(db_name: &str) {
        let postgresql_conn_url: String = DATABASE_URL.expose_secret().to_owned();

        let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
            .expect("Failed to parse PostgreSQL connection string");

        let mut connection = PgConnection::connect_with(&connection_options)
            .await
            .expect("Failed to connect to Postgres");

        // Kill any active connections to the database
        connection
            .execute(
                format!(
                    r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                    db_name
                )
                .as_str(),
            )
            .await
            .expect("Failed to drop the database.");

        // Drop the database
        connection
            .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
            .await
            .expect("Failed to drop the database.");
    }
    pub async fn clean_up(&mut self) {
        Self::delete_database(&self.db_name).await;
        self.clean_up_called = true
    }
}
pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::now_v7())
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            panic!("You must call clean_up()");
        }
    }
}
