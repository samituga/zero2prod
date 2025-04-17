use crate::api::helpers::{
    assert_is_redirected_to,
    spawn_app,
};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = spawn_app().await;
    let login_body = serde_json::json!( {
        "username": "username",
        "password": "password"
    });

    // Act
    let response = app.post_login(&login_body).await;

    // Assert
    let html_page = app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}

#[tokio::test]
async fn an_error_flash_message_is_not_set_after_failure_reload() {
    // Arrange
    let app = spawn_app().await;
    let login_body = serde_json::json!( {
        "username": "username",
        "password": "password"
    });

    // Act
    let response = app.post_login(&login_body).await;

    // Assert
    assert_is_redirected_to(&response, "/login");

    // load the login twice
    app.get_login_html().await;
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}
