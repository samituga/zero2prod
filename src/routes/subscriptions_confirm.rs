use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm(_parameters: web::Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let subscriber_id =
        match get_subscriber_id_from_token(&pool, &_parameters.subscription_token).await {
            Ok(id) => id,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

    match subscriber_id {
        None => HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            if confirm_subscriber(&pool, &subscriber_id).await.is_err() {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
    }
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(pool, subscription_token))]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
    SELECT subscriber_id FROM subscription_tokens
    WHERE subscription_token = $1
            "#,
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(pool, subscriber_id))]
async fn confirm_subscriber(pool: &PgPool, subscriber_id: &Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
            "#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
