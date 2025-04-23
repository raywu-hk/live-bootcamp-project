use crate::helpers::TestApp;
use axum::http::StatusCode;

#[tokio::test]
async fn verify_token_return_ok() {
    let app = TestApp::new().await;

    let response = app.get_verify_token().await;

    assert_eq!(response.status().as_u16(), StatusCode::OK);
}
