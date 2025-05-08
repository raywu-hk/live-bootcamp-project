//use test_macro::clean_up;
use crate::helpers::TestApp;
//#[clean_up]
#[tokio::test]
async fn root_returns_auth_ui() {
    let mut app = TestApp::new().await;

    let response = app.get_root().await;

    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");

    app.clean_up().await;
}
