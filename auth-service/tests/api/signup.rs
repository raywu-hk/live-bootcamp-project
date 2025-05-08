use crate::helpers::TestApp;
use auth_service::ErrorResponse;
use auth_service::routes::SignupResponse;
//use test_macro::clean_up;

//#[clean_up]
#[tokio::test]
async fn should_return_201_if_valid_input() {
    let mut app = TestApp::new().await;
    let random_email = TestApp::get_random_email();
    let valid_payload = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&valid_payload).await;
    let expected_response = SignupResponse {
        message: "User created successfully!".to_owned(),
    };
    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response
    );
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;
    let random_email = TestApp::get_random_email();
    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123",
        }),
        serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": "true"
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "failed for input {:?}",
            test_case
        );
    }
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    // The signup route should return a 400 HTTP status code if an invalid input is sent.
    // The input is considered invalid if:
    // - The email is empty or does not contain '@'
    // - The password is less than 8 characters
    let input = [
        serde_json::json!({
            "email": "test.com",
            "password": "8".repeat(8),
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "@test.com",
            "password": "8".repeat(8),
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "test@test.com",
            "password": "7".repeat(7),
            "requires2FA": true
        }),
    ];

    // Create an array of invalid inputs. Then, iterate through the array and
    // make HTTP calls to the signup route. Assert a 400 HTTP status code is returned.
    for i in input.iter() {
        let response = app.post_signup(i).await;
        assert_eq!(response.status().as_u16(), 400, "Failed for input: {:?}", i);

        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid credentials".to_owned()
        );
    }
    app.clean_up().await;
}
//#[clean_up]
#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    // Call the signup route twice. The second request should fail with a 409 HTTP status code
    let mut app = TestApp::new().await;
    let email_test_cases = [serde_json::json!({
        "email": "test@test.com",
        "password": "8".repeat(8),
        "requires2FA": true
    })];
    // Create an array of invalid inputs. Then, iterate through the array and
    // make HTTP calls to the signup route. Assert a 409 HTTP status code is returned.
    let response = app.post_signup(&email_test_cases[0]).await;
    assert_eq!(response.status().as_u16(), 201);

    let response = app.post_signup(&email_test_cases[0]).await;
    assert_eq!(response.status().as_u16(), 409);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );

    app.clean_up().await;
}
