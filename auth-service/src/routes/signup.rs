use crate::domain::{AuthAPIError, Email, Password, User};
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
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(&request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;
    // Create a new `User` instance using data in the `request`
    let user = User::new(email, password, request.requires_2fa);

    let mut user_store = state.user_store.write().await;
    if user_store.get_user(&user.email).await.is_ok() {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    // Add `user` to the `user_store`. Simply unwrap the returned `Result` enum type for now.
    if user_store.add_user(user).await.is_err() {
        return Err(AuthAPIError::UnexpectedError);
    }

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}
