use crate::EmailClient;
use crate::domain::{BannedTokenStore, TwoFACodeStore, UserStore};
use std::sync::Arc;
use tokio::sync::RwLock;

// Using a type alias to improve readability!
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync>>;
pub type BannedStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type TwoFACodeStoreType = Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>;
pub type EmailClientType = Arc<dyn EmailClient + Send + Sync>;
#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        banned_token_store: BannedStoreType,
        two_fa_code_store: TwoFACodeStoreType,
        email_client: EmailClientType,
    ) -> Self {
        AppState {
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }
}
