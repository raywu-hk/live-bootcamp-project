use crate::helpers::TestApp;
use reqwest::StatusCode;
use serde_json::json;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
    let payload = json!({
          "password": "password",
    });

    let response = app.post_login(&payload).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let signup_payload = json!({
        "email": "user@example.com",
        "password": "password",
        "requires2FA":false
    });
    let signup_result = app.post_signup(&signup_payload).await;

    assert_eq!(signup_result.status(), StatusCode::CREATED);

    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message.
    let login_payload = json!({
        "email": "@example.com",
        "password": "password",
    });
    let login_result = app.post_login(&login_payload).await;

    assert_eq!(login_result.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let signup_payload = json!({
        "email": "user@example.com",
        "password": "password",
        "requires2FA":false
    });
    let signup_result = app.post_signup(&signup_payload).await;

    assert_eq!(signup_result.status(), StatusCode::CREATED);
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let login_payload = json!({
        "email": "user@example.com",
        "password": "wrong password",
    });
    let login_result = app.post_login(&login_payload).await;

    assert_eq!(login_result.status(), StatusCode::UNAUTHORIZED);
}
