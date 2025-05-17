use crate::helpers::{TestApp, get_random_email};
use auth_service::Email;
use auth_service::routes::TwoFactorAuthResponse;
use auth_service::utils::JWT_COOKIE_NAME;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};
//use test_macro::clean_up;

//#[clean_up]
#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let mut app = TestApp::new().await;

    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();

    let signup_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let mut app = TestApp::new().await;
    let payload = json!({
          "password": "password",
    });

    let response = app.post_login(&payload).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

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
        "requires2FA": false
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
        "requires2FA": false
    });
    let signup_result = app.post_signup(&signup_payload).await;

    assert_eq!(signup_result.status(), StatusCode::CREATED);
    // Call the log-in route with incorrect credentials and assert
    // that a 401 HTTP status code is returned along with the appropriate error message.
    let login_payload = json!({
        "email": email.as_ref().expose_secret(),
        "password": "wrong password",
    });
    let login_result = app.post_login(&login_payload).await;

    assert_eq!(login_result.status(), StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
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

    // Define an expectation for the mock server
    Mock::given(path("/email")) // Expect an HTTP request to the "/email" path
        .and(method("POST")) // Expect the HTTP method to be POST
        .respond_with(ResponseTemplate::new(200)) // Respond with an HTTP 200 OK status
        .expect(1) // Expect this request to be made exactly once
        .mount(&app.email_server) // Mount this expectation on the mock email server
        .await; // Await the asynchronous operation to ensure the mock server is set up before proceeding

    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password"
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");
    assert_eq!(json_body.message, "2FA required".to_owned());

    //assert that `json_body.login_attempt_id` is stored inside `app.two_fa_code_store`
    let two_fa_tuple = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .expect("Could not get code");

    assert_eq!(
        json_body.login_attempt_id,
        two_fa_tuple.0.as_ref().expose_secret()
    );
    app.clean_up().await;
}
