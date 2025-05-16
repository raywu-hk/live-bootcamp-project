use crate::AppState;
use crate::domain::AuthAPIError;
use crate::utils::{JWT_COOKIE_NAME, validate_token};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use color_eyre::Result;
use color_eyre::eyre::ContextCompat;
#[tracing::instrument(name = "Logout", skip_all)]
pub async fn logout(
    state: State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    // Retrieve JWT cookie from the `CookieJar`
    // Return AuthAPIError::MissingToken if the cookie is not found
    let cookie = jar
        .get(JWT_COOKIE_NAME)
        .wrap_err("No Cookie found")
        .map_err(|_| AuthAPIError::MissingToken)?;

    let token = cookie.value().to_owned();
    // Validate JWT token by calling `validate_token` from the auth service.
    // If the token is valid, you can ignore the returned claims for now.
    // Return AuthAPIError::InvalidToken is validation fails.
    let _login_result = validate_token(&token, state.banned_token_store.clone())
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    state
        .banned_token_store
        .write()
        .await
        .add_token(&token)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    // remove token in cookie
    let updated_jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    Ok((updated_jar, StatusCode::OK))
}
