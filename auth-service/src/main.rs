use auth_service::utils::prod::APP_ADDRESS;
use auth_service::utils::{DATABASE_URL, POSTMARK_AUTH_TOKEN, REDIS_HOST_NAME, init_tracing, prod};
use auth_service::{
    AppState, Application, Email, PostgresUserStore, PostmarkEmailClient, RedisBannedTokenStore,
    RedisTwoFACodeStore, get_postgres_pool, get_redis_client,
};
use reqwest::Client;
use secrecy::SecretString;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to init tracing");
    let pg_pool = configure_postqresql().await;
    let redis_conn = Arc::new(RwLock::new(configure_redis()));
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn.clone())));
    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn.clone())));
    let email_client = Arc::new(configure_postmark_email_client());
    let app_state = AppState::new(
        user_store.clone(),
        banned_token_store.clone(),
        two_fa_code_store.clone(),
        email_client.clone(),
    );
    let app = Application::build(app_state, APP_ADDRESS)
        .await
        .expect("Failed to build app");
    app.run().await.expect("Failed to run app");
}

async fn configure_postqresql() -> PgPool {
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations scripts");
    pg_pool
}

fn configure_redis() -> redis::Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

fn configure_postmark_email_client() -> PostmarkEmailClient {
    let http_client = Client::builder()
        .timeout(prod::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(
        prod::email_client::BASE_URL.to_owned(),
        Email::parse(SecretString::from(prod::email_client::SENDER)).unwrap(),
        POSTMARK_AUTH_TOKEN.to_owned(),
        http_client,
    )
}
