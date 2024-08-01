use crate::create_cookie::create_cookie;
use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName, SubscriberPassword};
use crate::routes::error_chain_fmt;
use crate::utils::see_other;
use actix_web::{web, HttpResponse, ResponseError};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormData {
    full_name: String,
    email: String,
    password: String,
}
impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.full_name)?;
        let email = SubscriberEmail::parse(value.email)?;
        let password = SubscriberPassword::parse(value.password)?;
        Ok(Self {
            name,
            email,
            password,
        })
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SubscribeError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscribeError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_name = %form.full_name,
        subscriber_email = %form.email,
        subscriber_password = %form.password,
    )
)]
pub async fn 
register(
    form: web::Json<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SubscribeError> {

    

    let new_subscriber = NewSubscriber {
        name: SubscriberName::parse(form.0.full_name).expect("Name check failed"),
        email: SubscriberEmail::parse(form.0.email).expect("Email check failed"),
        password: SubscriberPassword::parse(form.0.password).expect("Password check failed"),
    };

    let cookie = create_cookie(&new_subscriber.name, &new_subscriber.password);

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let _subscriber_id = insert_user(&new_subscriber, &mut transaction)
        .await
        .context("Failed to insert new subscriber in the database.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;
    FlashMessage::error("Your account has been created.").send();
    Ok(HttpResponse::Ok().json(json!({"status":"success", "cookie":cookie})))
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_user(
    new_subscriber: &NewSubscriber,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    let query = sqlx::query!(
        r#"
    INSERT INTO users (id, account_name, account_password, account_email)
    VALUES ($1, $2, $3, $4)
            "#,
        subscriber_id,
        new_subscriber.name.as_ref(),
        new_subscriber.password.as_ref(),
        new_subscriber.email.as_ref(),
    );
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}
