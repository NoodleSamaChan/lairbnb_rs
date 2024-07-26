use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;
use sqlx::{Executor, PgPool, Postgres, Transaction};

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName, SubscriberPassword};

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
    password: String,
}
impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        let password = SubscriberPassword::parse(value.password)?;
        Ok(Self { email, name, password })
    }
}

#[allow(clippy::async_yields_async)]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
        subscriber_password = %form.password,
    )
)]
pub async fn register(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let subscriber_id = match insert_user(&new_subscriber, &mut transaction).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    };
    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
pub async fn insert_user(new_subscriber: &NewSubscriber, transaction: &mut Transaction<'_, Postgres>) -> Result<Uuid, sqlx::Error> {
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
