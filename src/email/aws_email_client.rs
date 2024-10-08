use crate::configuration::AwsSettings;
use crate::domain::Email;
use crate::email::email_client::{EmailClient, EmailClientProvider};
use aws_config::timeout::TimeoutConfig;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_sesv2::config::Credentials;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client;
use http::Uri;
use secrecy::{ExposeSecret, Secret};
use std::time::Duration;

pub type SesClient = Client;

#[async_trait::async_trait]
impl EmailClient for SesClient {
    async fn send_email(
        &self,
        sender_email: &Email,
        recipient_email: &Email,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<String, String> {
        let destination = Destination::builder()
            .to_addresses(recipient_email.as_ref())
            .build();

        let message = Message::builder()
            .subject(build_content(subject))
            .body(
                Body::builder()
                    .text(build_content(text_content))
                    .html(build_content(html_content))
                    .build(),
            )
            .build();

        let email_content = EmailContent::builder().simple(message).build();

        let result = self
            .send_email()
            .from_email_address(sender_email.as_ref())
            .destination(destination)
            .content(email_content)
            .send()
            .await;

        match result {
            Ok(out) => Ok(out.message_id.unwrap_or("success".to_string())),
            Err(e) => Err(format!("Failed to send email: {}", e)),
        }
    }
}

fn build_content(c: &str) -> Content {
    Content::builder().data(c).build().unwrap()
}

pub struct SesClientFactory {
    region: String,
    access_key_id: String,
    secret_access_key: Secret<String>,
    operation_timeout_secs: u64,
    operation_attempt_timeout_secs: u64,
    read_timeout_secs: u64,
    connect_timeout_secs: u64,
    endpoint_url: Option<Uri>,
}

impl SesClientFactory {
    pub fn new(settings: &AwsSettings) -> Self {
        Self {
            region: settings.region.clone(),
            access_key_id: settings.access_key_id.clone(),
            secret_access_key: settings.secret_access_key.clone(),
            operation_timeout_secs: settings.operation_timeout_secs,
            operation_attempt_timeout_secs: settings.operation_attempt_timeout_secs,
            read_timeout_secs: settings.read_timeout_secs,
            connect_timeout_secs: settings.connect_timeout_secs,
            endpoint_url: settings.endpoint_url.clone().map(|endpoint| {
                endpoint
                    .parse::<Uri>()
                    .expect("aws endpoint_url is not valid URI")
            }),
        }
    }
}

#[async_trait::async_trait]
impl EmailClientProvider<SesClient> for SesClientFactory {
    async fn email_client(&self) -> SesClient {
        let timeout_config = TimeoutConfig::builder()
            .operation_timeout(Duration::from_secs(self.operation_timeout_secs))
            .operation_attempt_timeout(Duration::from_secs(self.operation_attempt_timeout_secs))
            .read_timeout(Duration::from_secs(self.read_timeout_secs))
            .connect_timeout(Duration::from_secs(self.connect_timeout_secs))
            .build();

        let mut config_builder = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .credentials_provider(Credentials::new(
                &self.access_key_id,
                self.secret_access_key.expose_secret(),
                None,
                None,
                "manual",
            ))
            .region(Region::new(self.region.clone()))
            .timeout_config(timeout_config);

        if self.endpoint_url.is_some() {
            config_builder =
                config_builder.endpoint_url(self.endpoint_url.clone().unwrap().to_string());
        }

        let config = config_builder.load().await;
        SesClient::new(&config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_sesv2::operation::send_email::{SendEmailError, SendEmailOutput};
    use aws_sdk_sesv2::types::error::BadRequestException;
    use aws_smithy_mocks_experimental::{mock, mock_client, RuleMode};
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Paragraph;
    use fake::Fake;
    use tokio;

    #[tokio::test]
    async fn sends_email_with_correct_arguments() {
        // Arrange
        let sender_email = Email::parse(SafeEmail().fake::<String>()).unwrap();
        let recipient_email = Email::parse(SafeEmail().fake::<String>()).unwrap();
        let subject = Paragraph(1..10).fake::<String>();
        let text_content = Paragraph(1..10).fake::<String>();
        let html_content = format!("<p>{}</p>", text_content);

        let recipient_email_string = recipient_email.as_ref().to_string();

        let mock_send_email = mock!(SesClient::send_email)
            .match_requests(move |req| {
                req.destination()
                    .unwrap()
                    .to_addresses()
                    .contains(&recipient_email_string)
            })
            .then_output(|| {
                SendEmailOutput::builder()
                    .message_id("newsletter-email")
                    .build()
            });

        let aws_email_client: &dyn EmailClient =
            &mock_client!(aws_sdk_sesv2, RuleMode::Sequential, &[&mock_send_email]);

        // Act
        let result = aws_email_client
            .send_email(
                &sender_email,
                &recipient_email,
                &subject,
                &text_content,
                &html_content,
            )
            .await;

        // Assert
        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_client_returns_err() {
        // Arrange
        let sender_email = Email::parse(SafeEmail().fake::<String>()).unwrap();
        let recipient_email = Email::parse(SafeEmail().fake::<String>()).unwrap();
        let subject = Paragraph(1..10).fake::<String>();
        let text_content = Paragraph(1..10).fake::<String>();
        let html_content = format!("<p>{}</p>", text_content);

        let recipient_email_string = recipient_email.as_ref().to_string();

        let mock_send_email = mock!(SesClient::send_email)
            .match_requests(move |req| {
                req.destination()
                    .unwrap()
                    .to_addresses()
                    .contains(&recipient_email_string)
            })
            .then_error(|| {
                SendEmailError::BadRequestException(BadRequestException::builder().build())
            });

        let aws_email_client: &dyn EmailClient =
            &mock_client!(aws_sdk_sesv2, RuleMode::Sequential, &[&mock_send_email]);

        // Act
        let result = aws_email_client
            .send_email(
                &sender_email,
                &recipient_email,
                &subject,
                &text_content,
                &html_content,
            )
            .await;

        // Assert
        assert_err!(result);
    }
}
