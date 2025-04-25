use crate::domain::{BannableTokenStore, UserStores};
use std::sync::Arc;
use tokio::sync::RwLock;

// Using a type alias to improve readability!
pub type UserStoreType = Arc<RwLock<dyn UserStores + Send + Sync>>;
pub type BannedStoreType = Arc<RwLock<dyn BannableTokenStore + Send + Sync>>;
#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType, banned_token_store: BannedStoreType) -> Self {
        AppState {
            user_store,
            banned_token_store,
        }
    }
}
