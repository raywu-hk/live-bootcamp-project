use crate::helpers::TestApp;
use axum::http::StatusCode;

#[tokio::test]
async fn login_return_ok() {
    let app = TestApp::new().await;

    let response = app.get_login().await;

    assert_eq!(response.status(), StatusCode::OK);
}
