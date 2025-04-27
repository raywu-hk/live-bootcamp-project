use crate::helpers::TestApp;
use auth_service::Email;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let email = Email::parse("user@example.com").unwrap();
    let signup_payload = json!({
        "email": email.clone(),
        "password": "password",
        "requires2FA": true
    });
    let signup_result = app.post_signup(&signup_payload).await;
    assert_eq!(signup_result.status(), StatusCode::CREATED);

    let login_body = serde_json::json!({
        "email": "user@example.com",
        "password": "password"
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let (login_attempt_id, two_fa_code) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .unwrap();

    let malformed_two_fa_payload = json!({
        "email": email
    });

    let two_fa_result = app.post_verify_2fa(&malformed_two_fa_payload).await;

    assert_eq!(two_fa_result.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
