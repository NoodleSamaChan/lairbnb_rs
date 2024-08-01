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
    name = "Getting lairs from map coordinates",
    skip(pool)
)]
pub async fn lairs_based_on_coordinates(
    path: web::Path<(f64, f64, f64, f64)>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let path = path.into_inner();

    let mut transaction = pool
        .begin()
        .await
        .with_context(|| format!("Failed to launch the transaction"))
        .map_err(e500)?;

    fetch_lair_by_coordinates(path, &mut transaction)
        .await
        .context("Failed to store newsletter issue details")
        .map_err(e500)?;
    Ok(see_other("/admin/newsletters"))
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(transaction,)
)]
pub async fn fetch_lair_by_coordinates(
    path: (f64, f64, f64, f64),
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<HttpResponse, sqlx::Error> {
    let query = sqlx::query!(
        r#"
        SELECT id, title, image, lon, lat FROM rooms WHERE lat > $1 AND lat < $2 AND lon > $3 AND lon < $4
            "#,
        path.0,
        path.1,
        path.2,
        path.3,
    );
    transaction.execute(query).await?;
    Ok(HttpResponse::Ok().finish())
}