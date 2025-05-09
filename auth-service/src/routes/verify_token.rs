use crate::AppState;
use crate::domain::AuthAPIError;
use crate::utils::validate_token;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
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

    match state
        .banned_token_store
        .read()
        .await
        .contains_token(&token)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?
    {
        true => Err(AuthAPIError::InvalidToken),
        false => Ok(StatusCode::OK.into_response()),
    }
}
