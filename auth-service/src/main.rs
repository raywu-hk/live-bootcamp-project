use auth_service::utils::prod::APP_ADDRESS;
use auth_service::utils::{DATABASE_URL, REDIS_HOST_NAME, init_tracing};
use auth_service::{
    AppState, Application, MockEmailClient, PostgresUserStore, RedisBannedTokenStore,
    RedisTwoFACodeStore, get_postgres_pool, get_redis_client,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    init_tracing();
    let pg_pool = configure_postqresql().await;
    let redis_conn = Arc::new(RwLock::new(configure_redis()));
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn.clone())));
    let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn.clone())));
    let email_client = Arc::new(MockEmailClient {});
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
