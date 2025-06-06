use crate::helpers::{TestApp, get_random_email};
use auth_service::Email;
use auth_service::utils::JWT_COOKIE_NAME;
use axum::http::StatusCode;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
//use test_macro::clean_up;

//#[clean_up]
#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let verify_token_payload = json!({
         "malformed": ""
    });

    let response = app.post_verify_token(&verify_token_payload).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_200_valid_token() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password123",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_payload).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);

    let login_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password123",
    });
    let login_response = app.post_login(&login_payload).await;
    let jwt_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name().eq(JWT_COOKIE_NAME))
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let verify_token_payload = json!({
         "token": jwt_cookie.value(),
    });

    let response = app.post_verify_token(&verify_token_payload).await;

    assert_eq!(response.status(), StatusCode::OK);
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password123",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_payload).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);

    let login_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password123",
    });
    let login_response = app.post_login(&login_payload).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let invalid_token_payload = json!({
         "token": "invalid-token",
    });

    let response = app.post_verify_token(&invalid_token_payload).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_401_if_banned_token() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password123",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_payload).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);

    let login_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password123",
    });
    let login_response = app.post_login(&login_payload).await;
    let jwt_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name().eq(JWT_COOKIE_NAME))
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let verify_token_payload = json!({
         "token": jwt_cookie.value(),
    });

    let _logout_response = app.post_logout().await;
    let response = app.post_verify_token(&verify_token_payload).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}
