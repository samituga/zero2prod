use crate::api::helpers::{spawn_app, TestAppBootstrap};
use crate::aws_ses_rules::AwsRuleWrapper;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let body = "name=le guin&email=ursula_le_guin@gmail.com".to_string();

    let aws_rule_wrapper = AwsRuleWrapper::new_send_email_wrapper();
    let send_any_email_rule = aws_rule_wrapper.send_any_email_rule();

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

    let aws_rule_wrapper = AwsRuleWrapper::new_send_email_wrapper();
    let send_any_email_rule = aws_rule_wrapper.send_any_email_rule();

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
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let email = "ursula_le_guin@gmail.com";
    let body = format!("name=le guin&email={}", email);

    let aws_rule_wrapper = AwsRuleWrapper::new_send_email_wrapper();
    let send_any_email_rule = aws_rule_wrapper.send_any_email_rule();

    let app = TestAppBootstrap::builder()
        .aws_email_client_rules(&[send_any_email_rule])
        .spawn_app()
        .await;

    // Act
    let response = app.post_subscriptions(body).await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    let request = aws_rule_wrapper.expect_one_request();
    AwsRuleWrapper::assert_correct_destination(&request, email);
    AwsRuleWrapper::assert_correct_subject(&request, "Welcome");
    AwsRuleWrapper::assert_correct_body_text(&request, "Welcome to our newsletter!");

    let confirmation_links = app.extract_confirmation_links(&request);
    assert_eq!(confirmation_links.plain_text, confirmation_links.html)
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
