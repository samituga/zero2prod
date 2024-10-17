use crate::domain::{Email, SubscriberEmail};
use crate::routes::error_chain_fmt;
use std::fmt::{Debug, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct SendEmailRequest<'a> {
    pub to: &'a SubscriberEmail,
    pub subject: &'a str,
    pub html_content: &'a str,
    pub text_content: &'a str,
}

pub struct EmailService {
    sender_email: Email,
}

impl EmailService {
    pub fn new(sender: Email) -> Self {
        EmailService {
            sender_email: sender,
        }
    }

    pub async fn send_email(
        &self,
        email_client: &dyn EmailClient,
        send_email_request: SendEmailRequest<'_>,
    ) -> Result<(), EmailClientError> {
        email_client
            .send_email(&self.sender_email, send_email_request)
            .await
            .map_err(EmailClientError::SendEmailError)
    }
}

#[async_trait::async_trait]
pub trait EmailClient: Sync + Send {
    async fn send_email(
        &self,
        sender_email: &Email,
        request: SendEmailRequest<'_>,
    ) -> Result<(), anyhow::Error>;
}

#[async_trait::async_trait]
pub trait EmailClientProvider<T: EmailClient> {
    async fn email_client(&self) -> T;
}

#[derive(thiserror::Error)]
pub enum EmailClientError {
    #[error(transparent)]
    SendEmailError(#[from] anyhow::Error),
}

impl Debug for EmailClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_ok;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Paragraph;
    use fake::Fake;

    struct MockEmailClient<'a> {
        expected_sender_email: Email,
        expected_send_email_request: SendEmailRequest<'a>,
    }

    #[async_trait::async_trait]
    impl<'a> EmailClient for MockEmailClient<'a> {
        async fn send_email(
            &self,
            sender_email: &Email,
            send_email_request: SendEmailRequest<'_>,
        ) -> Result<(), anyhow::Error> {
            assert_eq!(*sender_email, self.expected_sender_email);
            assert_eq!(send_email_request, self.expected_send_email_request);
            Ok(())
        }
    }

    #[tokio::test]
    async fn calls_email_client_with_correct_arguments() {
        // Arrange
        let sender_email = Email::parse(SafeEmail().fake::<String>()).unwrap();
        let recipient_email = SubscriberEmail::parse(SafeEmail().fake::<String>()).unwrap();
        let subject = Paragraph(1..10).fake::<String>();
        let html_content = format!("<p>{}</p>", Paragraph(1..10).fake::<String>());
        let text_content = Paragraph(1..10).fake::<String>();

        let send_email_request = SendEmailRequest {
            to: &recipient_email,
            subject: &subject,
            html_content: &html_content,
            text_content: &text_content,
        };

        let mock_email_client = MockEmailClient {
            expected_sender_email: sender_email.clone(),
            expected_send_email_request: send_email_request.clone(),
        };

        let email_service = EmailService::new(sender_email.clone());

        // Act
        let result = email_service
            .send_email(&mock_email_client, send_email_request)
            .await;

        // Assert
        assert_ok!(&result);
    }
}
