use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client;

use crate::domain::SubscriberEmail;

#[async_trait::async_trait]
pub trait EmailSender {
    async fn send_email(
        &self,
        sender_email: &SubscriberEmail,
        recipient_email: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String>;
}

pub struct AwsSesEmailSender {
    ses_client: Client,
}

impl AwsSesEmailSender {
    pub fn new(ses_client: Client) -> Self {
        Self { ses_client }
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
    ) -> Result<(), String> {
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
            .ses_client
            .send_email()
            .from_email_address(sender_email.as_ref())
            .destination(destination)
            .content(email_content)
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
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
    ) -> Result<(), String> {
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
    use tokio;

    use crate::domain::SubscriberEmail;
    use crate::email_client::{AwsSesEmailSender, EmailSender};

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
        let aws_email_client = AwsSesEmailSender::new(client);

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
        let aws_email_client = AwsSesEmailSender::new(client);

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
