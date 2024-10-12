use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
#[allow(dead_code)] // TODO remove
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm(_parameters: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
