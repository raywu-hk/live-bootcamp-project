use crate::helpers::TestApp;
use axum::http::StatusCode;

#[tokio::test]
async fn logout_return_ok() {
    let app = TestApp::new().await;

    let response = app.get_logout().await;

    assert_eq!(response.status().as_u16(), StatusCode::OK);
}
