use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::routes::error_chain_fmt;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use secrecy::Secret;
use sqlx::PgPool;
use std::fmt::Debug;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub username: String,
    pub password: Secret<String>,
}

#[tracing::instrument(
    skip(form, pool),
    fields(username = %form.username, user_id = tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, LoginError> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };

    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;

    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    Ok(HttpResponse::SeeOther()
        .insert_header(("location", "/"))
        .finish())
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Invalid credentials.")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

impl Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::AuthError(_) => StatusCode::UNAUTHORIZED,
            LoginError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
