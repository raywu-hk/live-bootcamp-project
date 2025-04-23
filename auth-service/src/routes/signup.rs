use crate::domain::User;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}
pub async fn signup(
    // Use Axum's state extractor to pass in AppState
    state: State<AppState>,
    Json(request): Json<SignupRequest>,
) -> impl IntoResponse {
    // Create a new `User` instance using data in the `request`
    let user = User::new(request.email, request.password, request.requires_2fa);

    let mut user_store = state.user_store.write().await;

    // Add `user` to the `user_store`. Simply unwrap the returned `Result` enum type for now.
    user_store.add_user(user).unwrap();
    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    (StatusCode::CREATED, response)
}
