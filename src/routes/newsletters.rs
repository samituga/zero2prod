use crate::domain::SubscriberEmail;
use crate::email::email_client::{EmailClient, EmailService, SendEmailRequest};
use crate::routes::error_chain_fmt;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use std::fmt::{Debug, Formatter};

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl Debug for PublishError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_service: web::Data<EmailService>,
    email_client: web::Data<dyn EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let confirmed_subscribers = get_confirmed_subscribers(&pool).await?;

    for confirmed_subscriber in confirmed_subscribers {
        match confirmed_subscriber {
            Ok(confirmed_subscriber) => {
                let send_email_request = SendEmailRequest {
                    to: &confirmed_subscriber.email,
                    subject: &body.title,
                    html_content: &body.content.html,
                    text_content: &body.content.text,
                };

                email_service
                    .send_email(email_client.get_ref(), send_email_request)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to send newsletter issue to {}",
                            confirmed_subscriber.email
                        )
                    })?
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid"
                );
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

struct Row {
    email: String,
}

async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query_as!(
        Row,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?;
    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();
    Ok(confirmed_subscribers)
}
