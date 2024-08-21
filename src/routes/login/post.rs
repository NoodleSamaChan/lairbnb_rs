use crate::create_cookie::create_cookie;
use crate::domain::{SubscriberName, SubscriberPassword};
use crate::routes::error_chain_fmt;
use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;
use actix_web::{web, ResponseError};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use reqwest::StatusCode;
use secrecy::{ExposeSecret, Secret};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormData {
    full_name: String,
    password: Secret<String>,
}

#[tracing::instrument(
    skip(form, pool),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
// We are now injecting `PgPool` to retrieve stored credentials from the database
pub async fn login(
    form: web::Json<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, FetchError> {
    let credentials_check = check_user_exists(&form, pool).await?;

    if credentials_check == None {
        FlashMessage::info(
            "Account doesn't exist, please create an account before trying to login!",
        )
        .send();
        HttpResponse::SeeOther()
            .insert_header((LOCATION, "/registration"))
            .finish();
    }

    let cookie = create_cookie(
        &SubscriberName::parse(form.0.full_name).unwrap(),
        &SubscriberPassword::parse(form.0.password.expose_secret().to_string()).unwrap(),
    );

    Ok(HttpResponse::Ok().json(json!({"status":"success", "cookie":cookie})))
}

#[tracing::instrument(name = "checking if user exists", skip(form, pool))]
pub async fn check_user_exists(
    form: &web::Json<FormData>,
    pool: web::Data<PgPool>,
) -> Result<Option<Uuid>, FetchError> {
    let name = &form.0.full_name;
    let password = form.0.password.expose_secret();
    let query = sqlx::query!(
        r#"
    SELECT id FROM users WHERE account_name = $1 AND account_password = $2
            "#,
        name,
        password,
    )
    .fetch_one(&*pool.clone().into_inner())
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(Some(query.id))
}

#[derive(thiserror::Error)]
pub enum FetchError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for FetchError {
    fn status_code(&self) -> StatusCode {
        match self {
            FetchError::ValidationError(_) => StatusCode::BAD_REQUEST,
            FetchError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
