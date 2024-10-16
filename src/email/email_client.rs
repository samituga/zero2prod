use crate::domain::{Email, SubscriberEmail};
use crate::routes::error_chain_fmt;
use std::fmt::{Debug, Formatter};

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
        recipient_email: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailClientError> {
        email_client
            .send_email(
                &self.sender_email,
                recipient_email,
                subject,
                html_content,
                text_content,
            )
            .await
            .map_err(EmailClientError::SendEmailError)
    }
}

#[async_trait::async_trait]
pub trait EmailClient: Sync + Send {
    async fn send_email(
        &self,
        sender_email: &Email,
        recipient_email: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
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
    use async_trait::async_trait;
    use claims::assert_ok;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Paragraph;
    use fake::Fake;
    use mockall::{mock, predicate::*};

    mock! {
        pub EmailClient {}
        #[async_trait]
        impl EmailClient for EmailClient {
            async fn send_email(
                &self,
                sender_email: &Email,
                recipient_email: &SubscriberEmail,
                subject: &str,
                html_content: &str,
                text_content: &str,
            ) -> Result<(), anyhow::Error>;
        }
    }

    #[tokio::test]
    async fn calls_email_client_with_correct_arguments() {
        // Arrange
        let sender_email = Email::parse(SafeEmail().fake::<String>()).unwrap();
        let recipient_email = Email::parse(SafeEmail().fake::<String>()).unwrap();
        let subject = Paragraph(1..10).fake::<String>();
        let text_content = Paragraph(1..10).fake::<String>();
        let html_content = format!("<p>{}</p>", text_content);

        let mut mock_email_client = MockEmailClient::new();

        mock_email_client
            .expect_send_email()
            .with(
                eq(sender_email.clone()),
                eq(recipient_email.clone()),
                eq(subject.clone()),
                eq(html_content.clone()),
                eq(text_content.clone()),
            )
            .returning(|_, _, _, _, _| Ok(()));

        let email_service = EmailService::new(sender_email.clone());

        // Act
        let result = email_service
            .send_email(
                &mock_email_client,
                &recipient_email,
                subject.as_str(),
                html_content.as_str(),
                text_content.as_str(),
            )
            .await;

        // Assert
        assert_ok!(&result);
    }
}
