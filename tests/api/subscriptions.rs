use crate::api::helpers::{spawn_app, TestAppBootstrap};
use crate::aws_ses_rules::{send_any_email_rule, send_confirmation_email_with_a_link_rule};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let body = "name=le guin&email=ursula_le_guin@gmail.com".to_string();

    let send_any_email_rule = send_any_email_rule();

    let app = TestAppBootstrap::builder()
        .aws_email_client_rules(&[send_any_email_rule])
        .spawn_app()
        .await;

    // Act
    let response = app.post_subscriptions(body).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let email = "ursula_le_guin@gmail.com";
    let name = "le guin";
    let body = format!("name={}&email={}", name, email);

    let send_any_email_rule = send_any_email_rule();

    let app = TestAppBootstrap::builder()
        .aws_email_client_rules(&[send_any_email_rule])
        .spawn_app()
        .await;

    // Act
    let response = app.post_subscriptions(body).await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_with_a_link() {
    // Arrange
    let email = "ursula_le_guin@gmail.com";
    let body = format!("name=le guin&email={}", email);
    let message_id = "newsletter-email";

    let send_confirmation_email_with_a_link_rule =
        send_confirmation_email_with_a_link_rule(email.to_string());

    let app = TestAppBootstrap::builder()
        .aws_email_client_rules(&[send_confirmation_email_with_a_link_rule])
        .spawn_app()
        .await;

    // Act
    let response = app.post_subscriptions(body).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
    let response_body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(response_body["message_id"], message_id);
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_subscriptions(invalid_body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        // Act
        let response = app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a  400 Bad Request when the payload was {}.",
            description
        )
    }
}
