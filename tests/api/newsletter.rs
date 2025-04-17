use uuid::Uuid;

use crate::api::helpers::{
    spawn_app,
    ConfirmationLinks,
    TestApp,
};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter tittle",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });

    // Act
    let response = app.publish_newsletter(newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    app.aws_request_wrapper.expect_zero_requests();
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter tittle",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });

    // Act
    let response = app.publish_newsletter(newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    app.aws_request_wrapper.expect_one_request();
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>"
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.publish_newsletter(invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn non_existing_user_is_rejected() {
    // Arrange
    let app = spawn_app().await;

    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    // Act
    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(&username, Some(&password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn invalid_password_is_rejected() {
    // Arrange
    let app = spawn_app().await;
    let username = app.test_user.username;
    let password = Uuid::new_v4().to_string();
    assert_ne!(password, app.test_user.password);

    // Act
    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(&username, Some(&password))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "content": {
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le guin&email=ursula_le_guin@gmail.com";

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let request = app.aws_request_wrapper.expect_one_request_and_remove();
    app.extract_confirmation_links(&request)
}
