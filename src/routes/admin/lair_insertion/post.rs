use crate::{
    authentication::UserId, create_cookie::{extract_cookie, UserInfo}, routes::admin::{dashboard::get_username, password}, utils::{e500, see_other}
};
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LairInfo {
    title: String,
    description: String,
    image: String,
    lon: f64,
    lat: f64,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, lair_info)
)]
pub async fn insert_lair(
    pool: web::Data<PgPool>,
    lair_info: web::Json<LairInfo>,
    request: HttpRequest,
) -> Result<HttpResponse, InsertError> {
    let account_information = extract_cookie(request).unwrap();
    let room_id = Uuid::new_v4();

    let account_id = get_account_info(account_information, &pool).await?;

    let mut transaction = pool
        .begin()
        .await
        .with_context(|| format!("Failed to launch the transaction"))?;

    insert_lair_into_db(lair_info, &mut transaction, account_id, room_id)
        .await
        .context("Failed to store newsletter issue details")?;
    FlashMessage::info("You've added the lair successfully!").send();
    Ok(HttpResponse::Ok().json(json!({"status":"success"})))
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(lair_info, transaction, user_id, room_id)
)]
pub async fn insert_lair_into_db(
    lair_info: web::Json<LairInfo>,
    transaction: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    room_id: Uuid,
) -> Result<HttpResponse, sqlx::Error> {
    let query = sqlx::query!(
        r#"
    INSERT INTO rooms (id, title, image, description, lon, lat, room_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        user_id,
        lair_info.title,
        lair_info.image,
        lair_info.description,
        lair_info.lon,
        lair_info.lat,
        room_id,
    );
    transaction.execute(query).await?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Gatting account id",
    skip(account_information, pool)
)]
pub async fn get_account_info(
    account_information: UserInfo,
    pool: &web::Data<PgPool>,
) -> Result<Uuid, anyhow::Error> {
    let name: String = account_information.name;
    let password: String = account_information.password;
    let query = sqlx::query!(
        r#"
    SELECT id FROM users 
    WHERE account_name = $1 AND account_password = $2
            "#,
        name,
        password,
    ).fetch_one(&*pool.clone().into_inner())
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(query.id)
}


#[derive(thiserror::Error)]
pub enum InsertError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for InsertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
impl ResponseError for InsertError {
    fn status_code(&self) -> StatusCode {
        match self {
            InsertError::ValidationError(_) => StatusCode::BAD_REQUEST,
            InsertError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}