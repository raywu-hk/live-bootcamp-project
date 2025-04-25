use crate::helpers::TestApp;
use auth_service::utils::JWT_COOKIE_NAME;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let verify_token_payload = json!({
         "malformed": ""
    });

    let response = app.post_verify_token(&verify_token_payload).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;
    let signup_payload = serde_json::json!({
        "email": "email@example.com",
        "password": "password123",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_payload).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);

    let login_payload = json!({
        "email": "email@example.com",
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
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;
    let signup_payload = serde_json::json!({
        "email": "email@example.com",
        "password": "password123",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_payload).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);

    let login_payload = json!({
        "email": "email@example.com",
        "password": "password123",
    });
    let login_response = app.post_login(&login_payload).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let invalid_token_payload = json!({
         "token": "invalid-token",
    });

    let response = app.post_verify_token(&invalid_token_payload).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn should_return_401_if_banned_token() {
    let app = TestApp::new().await;
    let signup_payload = serde_json::json!({
        "email": "email@example.com",
        "password": "password123",
        "requires2FA": false
    });

    let signup_response = app.post_signup(&signup_payload).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);

    let login_payload = json!({
        "email": "email@example.com",
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
}
