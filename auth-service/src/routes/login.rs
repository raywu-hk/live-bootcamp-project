use crate::domain::{AuthAPIError, Email, Password};
use crate::{AppState, LoginAttemptId, TwoFACode, utils};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, response::IntoResponse};
use axum_extra::extract::CookieJar;
use color_eyre::Result;
use secrecy::{ExposeSecret, SecretString};
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
    pub password: SecretString,
}

// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

#[tracing::instrument(name = "Login", skip_all)]
pub async fn login(
    state: State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(SecretString::from(request.email)) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };
    let password = match Password::parse(SecretString::from(request.password)) {
        Ok(password) => password,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let user_store = state.user_store.read().await;

    if user_store.validate_user(&email, &password).await.is_err() {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e.into()))),
    };

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

#[tracing::instrument(name = "Handling 2fa", skip_all)]
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
    if let Err(e) = two_fa_store
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
    {
        return (jar, Err(AuthAPIError::UnexpectedError(e.into())));
    }
    //Send 2FA code via the email client. Return `AuthAPIError::UnexpectedError` if the operation fails.
    if let Err(e) = state
        .email_client
        .send_email(email, "2FA_Code", two_fa_code.as_ref().expose_secret())
        .await
    {
        return (jar, Err(AuthAPIError::UnexpectedError(e)));
    }
    // Return a TwoFactorAuthResponse. The message should be "2FA required".
    // The login attempt ID should be "123456". We will replace this hard-coded login attempt ID soon!
    let response = LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.as_ref().expose_secret().to_owned(),
    });
    (jar, Ok((StatusCode::PARTIAL_CONTENT, Json(response))))
}

#[tracing::instrument(name = "Handling no 2fa", skip_all)]
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
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e))),
    };

    let updated_jar = jar.add(auth_cookie);

    (
        updated_jar,
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
    )
}
