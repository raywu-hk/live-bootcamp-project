use crate::helpers::{TestApp, get_random_email};
use auth_service::Email;
use auth_service::utils::JWT_COOKIE_NAME;
use reqwest::{StatusCode, Url};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
//use test_macro::clean_up;

//#[clean_up]
#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
        "requires2FA": false
    });
    let mut app = TestApp::new().await;
    let signup_response = app.post_signup(&signup_body).await;
    assert_eq!(signup_response.status(), StatusCode::CREATED);

    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), StatusCode::OK);
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let email = get_random_email();
    let email = Email::parse(SecretString::from(email)).unwrap();
    let signup_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
        "requires2FA": false
    });
    let mut app = TestApp::new().await;
    let sign_response = app.post_signup(&signup_body).await;
    assert_eq!(sign_response.status(), StatusCode::CREATED);
    let login_body = json!({
        "email": email.as_ref().expose_secret(),
        "password": "password",
    });

    let login_response = app.post_login(&login_body).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), StatusCode::OK);
    //2nd time logout
    let logout_response = app.post_logout().await;
    assert_eq!(logout_response.status(), StatusCode::BAD_REQUEST);
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let mut app = TestApp::new().await;

    let response = app.post_logout().await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let mut app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    app.clean_up().await;
}
