use crate::{
    authentication::UserId,
    create_cookie::{extract_cookie, UserInfo},
    routes::{error_chain_fmt, get_account_info, get_username},
    utils::{e500, see_other},
};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, ResponseError};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RoomId {
    pub id: Uuid,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(path, pool)
)]

//#[get("/lair/{id}")]
pub async fn looking_at_lair(
    path: web::Path<RoomId>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, InsertError> {
    let path = path.id;

    let transaction = pool
        .begin()
        .await
        .with_context(|| format!("Failed to launch the transaction"))?;

    let found_lair: Lair = fetch_lair_by_id(path, pool.clone())
        .await
        .context("Failed to find lair")?;
    transaction
        .commit()
        .await
        .context("Failed to get lair on id.")?;
    Ok(HttpResponse::Ok().json(found_lair))
}

#[derive(Serialize)]
pub struct Lair {
    account_id: Uuid,
    title: String,
    description: String,
    image: String,
    lon: f64,
    lat: f64,
    #[serde(rename = "id")]
    room_id: Uuid,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(path, pool)
)]
pub async fn fetch_lair_by_id(path: Uuid, pool: web::Data<PgPool>) -> Result<Lair, anyhow::Error> {
    let query = sqlx::query_as!(
        Lair,
        r#"
        SELECT * FROM rooms WHERE room_id = $1 
            "#,
        path
    )
    .fetch_one(&*pool.clone().into_inner())
    .await
    .context("Failed to perform a query to retrieve a username.")?;

    Ok(query)

    //Ok(Lair { title: query.title, description: query.description, image: query.image, lon: query.lon, lat: query.lat })
}

//#[delete("/lair/{id}")]
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(path, pool)
)]
pub async fn deleting_lair(
    path: web::Path<RoomId>,
    pool: web::Data<PgPool>,
    request: HttpRequest,
) -> Result<HttpResponse, InsertError> {
    let account_information = extract_cookie(request.clone()).unwrap();
    let path = path.id;
    let user_id: Option<Uuid> = get_id_user(account_information, pool.clone()).await?;

    let mut transaction = pool
        .begin()
        .await
        .with_context(|| format!("Failed to launch the transaction"))?;

    delete_lair(path, user_id.unwrap(), &mut transaction)
        .await
        .context("Failed delete lair")?;
    transaction
        .commit()
        .await
        .context("Failed to delete lair.")?;
    FlashMessage::info("You've deleted the lair successfully").send();
    Ok(HttpResponse::Ok().json(json!({"status":"success"})))
}

#[tracing::instrument(name = "Deleting lair", skip(transaction,))]
pub async fn delete_lair(
    path: Uuid,
    user_id_as_uuid: Uuid,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<HttpResponse, anyhow::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM rooms WHERE room_id = $1 AND account_id = $2
            "#,
        path,
        user_id_as_uuid,
    );
    transaction.execute(query).await?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "checking if user exists", skip(account_information, pool))]
pub async fn get_id_user(
    account_information: UserInfo,
    pool: web::Data<PgPool>,
) -> Result<Option<Uuid>, anyhow::Error> {
    let name = &account_information.name;
    let password = &account_information.password;
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

/*pub fn error_chain_fmt(
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
} */
impl ResponseError for InsertError {
    fn status_code(&self) -> StatusCode {
        match self {
            InsertError::ValidationError(_) => StatusCode::BAD_REQUEST,
            InsertError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
