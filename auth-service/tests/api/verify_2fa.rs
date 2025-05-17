use crate::helpers::{TestApp, get_random_email};
use auth_service::Email;
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::utils::JWT_COOKIE_NAME;
use axum::http::StatusCode;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
//use test_macro::clean_up;

//#[clean_up]
#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
        "requires2FA": true
    });
    let signup_result = app.post_signup(&signup_payload).await;
    assert_eq!(signup_result.status(), StatusCode::CREATED);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password"
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let malformed_two_fa_payload = json!({
        "email": email.as_ref().expose_secret()
    });

    let two_fa_result = app.post_verify_2fa(&malformed_two_fa_payload).await;

    assert_eq!(two_fa_result.status(), StatusCode::UNPROCESSABLE_ENTITY);
    app.clean_up().await;
}

//#[clean_up]
#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
        "requires2FA": true
    });
    let signup_result = app.post_signup(&signup_payload).await;
    assert_eq!(signup_result.status(), StatusCode::CREATED);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password"
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let malformed_two_fa_payload = json!({
        "email": "@abc.com",
        "loginAttemptId": "invalid-attempt-id",
        "2FACode": "invalid"
    });

    let two_fa_result = app.post_verify_2fa(&malformed_two_fa_payload).await;

    assert_eq!(two_fa_result.status(), StatusCode::BAD_REQUEST);
    app.clean_up().await;
}

//#[clean_up]
#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
        "requires2FA": true
    });
    let signup_result = app.post_signup(&signup_payload).await;
    assert_eq!(signup_result.status(), StatusCode::CREATED);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password"
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let incorrect_two_fa_payload = json!({
        "email": email.as_ref().expose_secret(),
        "loginAttemptId": Uuid::now_v7().to_string(),
        "2FACode": "123456"
    });

    let two_fa_result = app.post_verify_2fa(&incorrect_two_fa_payload).await;

    assert_eq!(two_fa_result.status(), StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}

//#[clean_up]
#[tokio::test]
async fn should_return_401_if_old_code() {
    // Call login twice. Then, attempt to call verify-fa with the 2FA code from the first login request. This should fail.
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
        "requires2FA": true
    });
    let signup_result = app.post_signup(&signup_payload).await;
    assert_eq!(signup_result.status(), StatusCode::CREATED);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password"
    });

    let first_login_response = app.post_login(&login_body).await;
    assert_eq!(first_login_response.status(), StatusCode::PARTIAL_CONTENT);

    let first_login_2fa_code = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .unwrap();

    let second_login_response = app.post_login(&login_body).await;
    assert_eq!(second_login_response.status(), StatusCode::PARTIAL_CONTENT);

    let reuse_two_fa_payload = json!({
        "email": email.as_ref().expose_secret(),
        "loginAttemptId": first_login_2fa_code.0.as_ref().expose_secret(),
        "2FACode": first_login_2fa_code.1.as_ref().expose_secret()
    });

    let two_fa_result = app.post_verify_2fa(&reuse_two_fa_payload).await;

    assert_eq!(two_fa_result.status(), StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}

//#[clean_up]
#[tokio::test]
async fn should_return_200_if_correct_code() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
        "requires2FA": true
    });
    let signup_result = app.post_signup(&signup_payload).await;
    assert_eq!(signup_result.status(), StatusCode::CREATED);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password"
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .unwrap();
    let two_fa_auth_response = response.json::<TwoFactorAuthResponse>().await.unwrap();
    let two_fa_payload = json!({
        "email": email.as_ref().expose_secret(),
        "loginAttemptId": two_fa_auth_response.login_attempt_id,
        "2FACode": code_tuple.1.as_ref().expose_secret()
    });

    let two_fa_result = app.post_verify_2fa(&two_fa_payload).await;
    let response_cookie = two_fa_result
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .unwrap();

    assert_eq!(two_fa_result.status(), StatusCode::OK);
    assert_eq!(response_cookie.name(), JWT_COOKIE_NAME);
    assert!(!response_cookie.value().is_empty());

    app.clean_up().await;
}

//#[clean_up]
#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
        "requires2FA": true
    });
    let signup_result = app.post_signup(&signup_payload).await;
    assert_eq!(signup_result.status(), StatusCode::CREATED);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password"
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let code_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .unwrap();
    let two_fa_auth_response = response.json::<TwoFactorAuthResponse>().await.unwrap();
    let two_fa_payload = json!({
        "email": email.as_ref().expose_secret(),
        "loginAttemptId": two_fa_auth_response.login_attempt_id,
        "2FACode": code_tuple.1.as_ref().expose_secret()
    });

    let two_fa_result = app.post_verify_2fa(&two_fa_payload).await;
    let response_cookie = two_fa_result
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .unwrap();
    assert_eq!(two_fa_result.status(), StatusCode::OK);
    assert_eq!(response_cookie.name(), JWT_COOKIE_NAME);
    assert!(!response_cookie.value().is_empty());

    let two_fa_result = app.post_verify_2fa(&two_fa_payload).await;
    assert_eq!(two_fa_result.status(), StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}
