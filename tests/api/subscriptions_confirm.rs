use crate::api::helpers::{spawn_app, TestAppBootstrap};
use crate::aws_ses_rules::AwsRuleWrapper;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn the_link_by_subscribe_returns_a_200_if_called() {
    // Arrange
    let email = "ursula_le_guin@gmail.com";
    let body = format!("name=le guin&email={}", email);

    let aws_rule_wrapper = AwsRuleWrapper::new_send_email_wrapper();
    let send_any_email_rule = aws_rule_wrapper.send_any_email_rule();

    let app = TestAppBootstrap::builder()
        .aws_email_client_rules(&[send_any_email_rule])
        .spawn_app()
        .await;

    app.post_subscriptions(body).await;

    let send_confirmation_email_with_a_link_request = aws_rule_wrapper.expect_one_request();
    let confirmation_links =
        app.extract_confirmation_links(&send_confirmation_email_with_a_link_request);

    // Act
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    // Arrange
    let email = "ursula_le_guin@gmail.com";
    let body = format!("name=le guin&email={}", email);

    let aws_rule_wrapper = AwsRuleWrapper::new_send_email_wrapper();
    let send_any_email_rule = aws_rule_wrapper.send_any_email_rule();

    let app = TestAppBootstrap::builder()
        .aws_email_client_rules(&[send_any_email_rule])
        .spawn_app()
        .await;

    app.post_subscriptions(body).await;

    let send_confirmation_email_with_a_link_request = aws_rule_wrapper.expect_one_request();

    let confirmation_links =
        app.extract_confirmation_links(&send_confirmation_email_with_a_link_request);

    // Act
    reqwest::get(confirmation_links.html).await.unwrap();

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}
