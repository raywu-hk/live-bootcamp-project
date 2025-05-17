use crate::AppState;
use crate::domain::AuthAPIError;
use crate::utils::validate_token;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use secrecy::SecretString;
use serde::Deserialize;
#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

#[tracing::instrument(name = "Verify Token", skip_all)]
pub async fn verify_token(
    state: State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let token = SecretString::from(request.token);
    validate_token(&token, state.banned_token_store.clone())
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    match state
        .banned_token_store
        .read()
        .await
        .contains_token(&token)
        .await
    {
        Ok(true) => Err(AuthAPIError::InvalidToken),
        Ok(false) => Ok(StatusCode::OK.into_response()),
        Err(_) => Err(AuthAPIError::UnexpectedError(eyre!("oh no"))),
    }
}
