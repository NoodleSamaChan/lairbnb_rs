use crate::{
    authentication::UserId, routes::get_username, utils::{e500, see_other}
};
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use serde::Deserialize;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool)
)]
pub async fn looking_at_lair(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let path = path.into_inner();

    let mut transaction = pool
        .begin()
        .await
        .with_context(|| format!("Failed to launch the transaction"))
        .map_err(e500)?;

    fetch_lair_by_id(path, &mut transaction)
        .await
        .context("Failed to store newsletter issue details")
        .map_err(e500)?;
    Ok(see_other("/admin/newsletters"))
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(path, transaction,)
)]
pub async fn fetch_lair_by_id(
    path: Uuid,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<HttpResponse, sqlx::Error> {
    let query = sqlx::query!(
        r#"
        SELECT * FROM rooms WHERE room_id = $1
            "#,
        path
    );
    transaction.execute(query).await?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool)
)]
pub async fn deleting_lair(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let path = path.into_inner();
    let user_id = user_id.into_inner();
    let user_id_as_uuid: Uuid = *user_id;

    let mut transaction = pool
        .begin()
        .await
        .with_context(|| format!("Failed to launch the transaction"))
        .map_err(e500)?;

    delete_lair(path, user_id_as_uuid, &mut transaction)
        .await
        .context("Failed to store newsletter issue details")
        .map_err(e500)?;
    FlashMessage::info("You've added the lair successfully!").send();
    Ok(see_other("/admin/newsletters"))
}

#[tracing::instrument(
    name = "Deleting lair",
    skip(transaction,)
)]
pub async fn delete_lair(
    path: Uuid,
    user_id_as_uuid: Uuid,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<HttpResponse, sqlx::Error> {
    let query = sqlx::query!(
        r#"
        DELETE FROM rooms WHERE room_id = $1 AND id = $2
            "#,
        path,
        user_id_as_uuid,
    );
    transaction.execute(query).await?;
    Ok(HttpResponse::Ok().finish())
}