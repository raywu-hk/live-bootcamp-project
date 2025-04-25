use crate::domain::AuthAPIError;
use crate::utils::validate_token;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

pub async fn verify_token(
    state: State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let token = request.token;
    validate_token(&token)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    let banned_token_store = state.banned_token_store.read().await;
    if banned_token_store.is_banned(&token).await {
        return Err(AuthAPIError::InvalidToken);
    }
    Ok(StatusCode::OK.into_response())
}
