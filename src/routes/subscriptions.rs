use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email::email_client::{EmailClient, EmailService};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_service, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_service: web::Data<EmailService>,
    email_client: web::Data<dyn EmailClient>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let insert_subscriber_result = insert_subscriber(&pool, &new_subscriber).await;
    if insert_subscriber_result.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    let send_confirmation_email_result =
        send_confirmation_email(new_subscriber, &email_service, email_client.get_ref()).await;

    match send_confirmation_email_result {
        Ok(message_id) => HttpResponse::Ok().json(json!({"message_id": message_id})),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

async fn send_confirmation_email(
    new_subscriber: NewSubscriber,
    email_service: &EmailService,
    email_client: &dyn EmailClient,
) -> Result<String, String> {
    let confirmation_link = "https://there-is-no-such-domain.com/subscriptions/confirm";

    let html_content = &format!(
        "Welcome to our newsletter!<br />\
                Click <a href=\"{}\"here</a> to confirm your subscription.",
        confirmation_link
    );

    let text_content = &format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    email_service
        .send_email(
            email_client,
            &new_subscriber.email,
            "Welcome",
            html_content,
            text_content,
        )
        .await
}
