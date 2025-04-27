use auth_service::utils::prod::APP_ADDRESS;
use auth_service::{
    AppState, Application, HashSetBannedTokenStore, HashmapTwoFACodeStore, HashmapUserStore,
    MockEmailClient,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let banned_token_store = Arc::new(RwLock::new(HashSetBannedTokenStore::default()));
    let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
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
