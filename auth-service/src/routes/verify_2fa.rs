use crate::{Email, LoginAttemptId, TwoFACode};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;

// Implement the Verify2FARequest struct. See the verify-2fa route contract in step 1 for the expected JSON body.
#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: Email,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: LoginAttemptId,
    #[serde(rename = "2FACode")]
    two_fa_code: TwoFACode,
}
pub async fn verify_2fa(Json(request): Json<Verify2FARequest>) -> impl IntoResponse {
    StatusCode::OK.into_response()
}
