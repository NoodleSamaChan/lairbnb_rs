use crate::{
    authentication::UserId,
    routes::admin::dashboard::get_username,
    utils::{e500, see_other},
};
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use serde::Deserialize;
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
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let _username = get_username(*user_id, &pool).await.map_err(e500)?;
    let room_id = Uuid::new_v4();

    let mut transaction = pool
        .begin()
        .await
        .with_context(|| format!("Failed to launch the transaction"))
        .map_err(e500)?;

    insert_lair_into_db(lair_info, &mut transaction, user_id, room_id)
        .await
        .context("Failed to store newsletter issue details")
        .map_err(e500)?;
    FlashMessage::info("You've added the lair successfully!").send();
    Ok(see_other("/admin/newsletters"))
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(lair_info, transaction, user_id, room_id)
)]
pub async fn insert_lair_into_db(
    lair_info: web::Json<LairInfo>,
    transaction: &mut Transaction<'_, Postgres>,
    user_id: UserId,
    room_id: Uuid,
) -> Result<HttpResponse, sqlx::Error> {
    let uuid_id: Uuid = *user_id;
    let query = sqlx::query!(
        r#"
    INSERT INTO rooms (id, title, image, description, lon, lat, room_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        uuid_id,
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
