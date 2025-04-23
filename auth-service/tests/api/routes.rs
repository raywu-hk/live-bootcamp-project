use crate::helpers::TestApp;
use reqwest::StatusCode;

// Tokio's test macro is used to run the test in an async environment
#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;
    let response = app.get_root().await;
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}

// For now, simply assert that each route returns a 200 HTTP status code.
#[tokio::test]
async fn login_return_ok() {
    let app = TestApp::new().await;
    let response = app.get_login().await;
    assert_eq!(response.status(), StatusCode::OK);
}
#[tokio::test]
async fn signup_return_ok() {
    let app = TestApp::new().await;
    let response = app.get_signup().await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn logout_return_ok() {
    let app = TestApp::new().await;
    let response = app.get_logout().await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn verify_2fa_return_ok() {
    let app = TestApp::new().await;
    let response = app.get_verify_2fa().await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn verify_token_return_ok() {
    let app = TestApp::new().await;
    let response = app.get_verify_token().await;
    assert_eq!(response.status(), StatusCode::OK);
}
