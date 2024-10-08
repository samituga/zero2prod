use crate::helpers::{spawn_app, TestAppBootstrap};
use aws_sdk_sesv2::operation::send_email::SendEmailOutput;
use aws_smithy_mocks_experimental::mock;
use zero2prod::email::aws_email_client::SesClient;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let email = "ursula_le_guin@gmail.com";
    let name = "le guin";
    let body = format!("name={}&email={}", name, email);

    let app = spawn_app().await;

    // Act
    let response = app.post_subscriptions(body).await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);
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

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let email = "ursula_le_guin@gmail.com";
    let name = "le guin";
    let message_id = "newsletter-email";
    let body = format!("name={}&email={}", name, email);

    let mock_send_email = mock!(SesClient::send_email)
        .match_requests(move |req| {
            req.destination()
                .unwrap()
                .to_addresses()
                .contains(&email.to_string())
        })
        .then_output(move || SendEmailOutput::builder().message_id(message_id).build());

    let app = TestAppBootstrap::builder()
        .aws_email_client_rules(&[mock_send_email])
        .spawn_app()
        .await;

    // Act
    let response = app.post_subscriptions(body).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
    let response_body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(response_body["message_id"], message_id);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);
}
