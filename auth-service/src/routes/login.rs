use crate::domain::{AuthAPIError, Email, Password};
use crate::{utils, AppState, LoginAttemptId, TwoFACode};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

// The login route can return 2 possible success responses.
// This enum models each response!
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

pub async fn login(
    state: State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(&request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };
    let password = match Password::parse(&request.password) {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let user_store = state.user_store.read().await;

    if user_store.validate_user(&email, &password).await.is_err() {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();
    let mut two_fa_store = state.two_fa_code_store.write().await;
    if two_fa_store
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }
    // Return a TwoFactorAuthResponse. The message should be "2FA required".
    // The login attempt ID should be "123456". We will replace this hard-coded login attempt ID soon!
    let response = LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.as_ref().to_string(),
    });
    (jar, Ok((StatusCode::PARTIAL_CONTENT, Json(response))))
}

async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    // Call the generate_auth_cookie function defined in the auth module.
    // If the function call fails, return AuthAPIError::UnexpectedError.
    let auth_cookie = match utils::generate_auth_cookie(email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    let updated_jar = jar.add(auth_cookie);

    (
        updated_jar,
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
    )
}
