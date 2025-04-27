use crate::utils::generate_auth_cookie;
use crate::{AppState, AuthAPIError, Email, LoginAttemptId, TwoFACode};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

// Implement the Verify2FARequest struct. See the verify-2fa route contract in step 1 for the expected JSON body.
#[derive(Serialize, Deserialize)]
pub struct Verify2FARequest {
    pub email: Email,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: String,
    #[serde(rename = "2FACode")]
    two_fa_code: String,
}

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let email =
        Email::parse(request.email.as_ref()).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let login_attempt_id = LoginAttemptId::parse(request.login_attempt_id.clone())
        .map_err(|_| AuthAPIError::InvalidCredentials)?;
    let two_fa_code =
        TwoFACode::parse(request.two_fa_code).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let mut two_fa_code_store = state.two_fa_code_store.write().await;

    // Call `two_fa_code_store.get_code`. If the call fails
    // return a `AuthAPIError::IncorrectCredentials`.
    let code_tuple = two_fa_code_store
        .get_code(&email)
        .await
        .map_err(|_| AuthAPIError::IncorrectCredentials)?;

    // Validate that the `login_attempt_id` and `two_fa_code`
    // in the request body matches values in the `code_tuple`.
    // If not, return an ` AuthAPIError::IncorrectCredentials `.
    match code_tuple {
        (store_login_attempt_id, store_two_fa_code)
            if store_login_attempt_id == login_attempt_id && store_two_fa_code == two_fa_code =>
        {
            two_fa_code_store
                .remove_code(&email)
                .await
                .map_err(|_| AuthAPIError::UnexpectedError)?;
            let cookie = generate_auth_cookie(&email).map_err(|_| AuthAPIError::UnexpectedError)?;
            let updated_jar = jar.add(cookie);
            Ok((updated_jar, StatusCode::OK.into_response()))
        }
        _ => Err(AuthAPIError::IncorrectCredentials),
    }
}
