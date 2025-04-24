use crate::domain::{AuthAPIError, User};
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
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = request.email;
    let password = request.password;
    if is_email_invalid(&email) || is_password_invalue(&password) {
        return Err(AuthAPIError::InvalidCredentials);
    }
    // Create a new `User` instance using data in the `request`
    let user = User::new(email, password, request.requires_2fa);

    let mut user_store = state.user_store.write().await;
    if user_store.get_user(&user.email).is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    // Add `user` to the `user_store`. Simply unwrap the returned `Result` enum type for now.
    if user_store.add_user(user).is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

fn is_email_invalid(email: &str) -> bool {
    if email.is_empty() {
        return true;
    }
    if !email.contains("@") {
        return true;
    }
    if email.starts_with('@') {
        return true;
    }
    false
}

fn is_password_invalue(password: &str) -> bool {
    if password.len() < 8 {
        return true;
    }
    false
}
