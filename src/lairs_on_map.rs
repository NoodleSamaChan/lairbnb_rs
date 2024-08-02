use crate::{authentication::UserId, routes::error_chain_fmt};
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LairsOnMap {
    br_lat: f64,
    br_lng: f64,
    tl_lat: f64,
    tl_lng: f64,
    search: Option<String>,
}

#[tracing::instrument(name = "Getting lairs from map coordinates", skip(pool))]
pub async fn lairs_based_on_coordinates(
    info: web::Query<LairsOnMap>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, InsertError> {
    let transaction = pool
        .begin()
        .await
        .with_context(|| format!("Failed to launch the transaction"))?;

    let search = info.search.clone();
    println!("{:#?}", search);
    if search != None {
        let result = fetch_lair_by_coordinates_with_search(info, pool, search.unwrap()).await?;
        transaction
            .commit()
            .await
            .context("Failed to get lair on map.")?;
        return Ok(HttpResponse::Ok().json(result));
    } else {
        let result = fetch_lair_by_coordinates_without_search(info, pool).await?;
        transaction
            .commit()
            .await
            .context("Failed to get lair on map.")?;
        return Ok(HttpResponse::Ok().json(result));
    }
}

#[derive(Serialize)]
pub struct LairFetched {
    account_id: Uuid,
    title: String,
    image: String,
    lon: f64,
    lat: f64,
    #[serde(rename = "id")]
    room_id: Uuid,
}

#[tracing::instrument(name = "Fetching lairs with search", skip(info, pool, search))]
pub async fn fetch_lair_by_coordinates_with_search(
    info: web::Query<LairsOnMap>,
    pool: web::Data<PgPool>,
    search: String,
) -> Result<LairFetched, anyhow::Error> {
    let query_with_search = sqlx::query_as!(
            LairFetched,
            r#"
            SELECT account_id, title, image, lon, lat, room_id FROM rooms WHERE lat > $1 AND lat < $2 AND lon > $3 AND lon < $4 AND title LIKE $5
                "#,
            info.br_lat,
            info.tl_lat,
            info.tl_lng,
            info.br_lng,
            search
        ).fetch_one(&*pool.clone().into_inner())
        .await
        .context("Failed to perform a query to retrieve a specific lair.")?;
    return Ok(query_with_search);
}

#[tracing::instrument(name = "Fetching lairs without search", skip(info, pool,))]
pub async fn fetch_lair_by_coordinates_without_search(
    info: web::Query<LairsOnMap>,
    pool: web::Data<PgPool>,
) -> Result<Vec<LairFetched>, anyhow::Error> {
    let query_without_search = sqlx::query_as!(
            LairFetched,
            r#"
            SELECT account_id, title, image, lon, lat, room_id FROM rooms WHERE lat > $1 AND lat < $2 AND lon > $3 AND lon < $4
                "#,
            info.br_lat,
            info.tl_lat,
            info.tl_lng,
            info.br_lng,
            ).fetch_all(&*pool.clone().into_inner())
            .await
            .context("Failed to perform a query to retrieve a username.")?;
    return Ok(query_without_search);
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

impl ResponseError for InsertError {
    fn status_code(&self) -> StatusCode {
        match self {
            InsertError::ValidationError(_) => StatusCode::BAD_REQUEST,
            InsertError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
