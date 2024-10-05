use crate::configuration::AwsSettings;
use crate::domain::SubscriberEmail;
use aws_config::timeout::TimeoutConfig;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_sesv2::config::Credentials;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client;
use secrecy::{ExposeSecret, Secret};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;

pub struct SesClientFactory {
    region: String,
    access_key_id: String,
    secret_access_key: Secret<String>,
    operation_timeout_secs: u64,
    operation_attempt_timeout_secs: u64,
    read_timeout_secs: u64,
    connect_timeout_secs: u64,
    ses_client_singleton: Arc<OnceCell<Client>>,
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
            ses_client_singleton: Arc::new(OnceCell::new()),
        }
    }
}

#[async_trait::async_trait]
impl SesClientProvider for SesClientFactory {
    async fn ses_client(&self) -> Client {
        let cache = Arc::clone(&self.ses_client_singleton);
        cache
            .get_or_init(|| async {
                let timeout_config = TimeoutConfig::builder()
                    .operation_timeout(Duration::from_secs(self.operation_timeout_secs))
                    .operation_attempt_timeout(Duration::from_secs(
                        self.operation_attempt_timeout_secs,
                    ))
                    .read_timeout(Duration::from_secs(self.read_timeout_secs))
                    .connect_timeout(Duration::from_secs(self.connect_timeout_secs))
                    .build();

                let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
                    .credentials_provider(Credentials::new(
                        &self.access_key_id,
                        self.secret_access_key.expose_secret(),
                        None,
                        None,
                        "manual",
                    ))
                    .region(Region::new(self.region.clone()))
                    .timeout_config(timeout_config)
                    .load()
                    .await;

                Client::new(&config)
            })
            .await
            .clone()
    }
}

#[async_trait::async_trait]
pub trait EmailSender {
    async fn send_email(
        &self,
        sender_email: &SubscriberEmail,
        recipient_email: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<String, String>;
}

#[async_trait::async_trait]
pub trait SesClientProvider {
    async fn ses_client(&self) -> Client;
}

pub struct AwsSesEmailSender {
    ses_client_provider: Arc<dyn SesClientProvider + Send + Sync>,
}

impl AwsSesEmailSender {
    pub fn new(ses_client_provider: Arc<dyn SesClientProvider + Send + Sync>) -> Self {
        Self {
            ses_client_provider,
        }
    }
}

#[async_trait::async_trait]
impl EmailSender for AwsSesEmailSender {
    async fn send_email(
        &self,
        sender_email: &SubscriberEmail,
        recipient_email: &SubscriberEmail,
        subject: &str,
        text_content: &str,
        html_content: &str,
    ) -> Result<String, String> {
        let ses_client = self.ses_client_provider.ses_client().await;
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

        let result = ses_client
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

pub struct EmailService<T: EmailSender + Send + Sync> {
    email_sender: T,
    sender_email: SubscriberEmail,
}

impl<T: EmailSender + Send + Sync> EmailService<T> {
    pub fn new(email_sender: T, sender: SubscriberEmail) -> Self {
        EmailService {
            email_sender,
            sender_email: sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient_email: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<String, String> {
        self.email_sender
            .send_email(
                &self.sender_email,
                recipient_email,
                subject,
                html_content,
                text_content,
            )
            .await
    }
}

fn build_content(c: &str) -> Content {
    Content::builder().data(c).build().unwrap()
}

#[cfg(test)]
mod tests {
    use aws_sdk_sesv2::operation::send_email::{SendEmailError, SendEmailOutput};
    use aws_sdk_sesv2::types::error::BadRequestException;
    use aws_sdk_sesv2::Client;
    use aws_smithy_mocks_experimental::{mock, mock_client, RuleMode};
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Paragraph;
    use fake::Fake;
    use std::sync::Arc;
    use tokio;

    use crate::domain::SubscriberEmail;
    use crate::email_client::{AwsSesEmailSender, EmailSender, SesClientProvider};

    struct MockSesClientProvider {
        client: Client,
    }

    #[async_trait::async_trait]
    impl SesClientProvider for MockSesClientProvider {
        async fn ses_client(&self) -> Client {
            self.client.clone()
        }
    }

    #[tokio::test]
    async fn sends_email_with_correct_arguments() {
        // Arrange
        let sender_email = SubscriberEmail::parse(SafeEmail().fake::<String>()).unwrap();
        let recipient_email = SubscriberEmail::parse(SafeEmail().fake::<String>()).unwrap();
        let subject = Paragraph(1..10).fake::<String>();
        let text_content = Paragraph(1..10).fake::<String>();
        let html_content = format!("<p>{}</p>", text_content);

        let recipient_email_string = recipient_email.as_ref().to_string();

        let mock_send_email = mock!(Client::send_email)
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

        let client = mock_client!(aws_sdk_sesv2, RuleMode::Sequential, &[&mock_send_email]);
        let mock_ses_client_provider = Arc::new(MockSesClientProvider { client });
        let aws_email_client = AwsSesEmailSender::new(mock_ses_client_provider);

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
        let sender_email = SubscriberEmail::parse(SafeEmail().fake::<String>()).unwrap();
        let recipient_email = SubscriberEmail::parse(SafeEmail().fake::<String>()).unwrap();
        let subject = Paragraph(1..10).fake::<String>();
        let text_content = Paragraph(1..10).fake::<String>();
        let html_content = format!("<p>{}</p>", text_content);

        let recipient_email_string = recipient_email.as_ref().to_string();

        let mock_send_email = mock!(Client::send_email)
            .match_requests(move |req| {
                req.destination()
                    .unwrap()
                    .to_addresses()
                    .contains(&recipient_email_string)
            })
            .then_error(|| {
                SendEmailError::BadRequestException(BadRequestException::builder().build())
            });

        let client = mock_client!(aws_sdk_sesv2, RuleMode::Sequential, &[&mock_send_email]);
        let mock_ses_client_provider = Arc::new(MockSesClientProvider { client });
        let aws_email_client = AwsSesEmailSender::new(mock_ses_client_provider);

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
