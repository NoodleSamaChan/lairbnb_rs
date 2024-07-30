use super::error_chain_fmt;
use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    domain::{LairDescription, LairImage, LairLat, LairLon, LairTitle, NewLair},
};
use actix_web::http::StatusCode;
use actix_web::{
    http::header::{HeaderMap, HeaderValue},
    web, HttpRequest, HttpResponse, ResponseError,
};
use anyhow::Context;
use base64::Engine;
use secrecy::Secret;
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
impl TryFrom<LairInfo> for NewLair {
    type Error = String;

    fn try_from(value: LairInfo) -> Result<Self, Self::Error> {
        let title = LairTitle::parse(value.title)?;
        let description = LairDescription::parse(value.description)?;
        let image = LairImage::parse(value.image)?;
        let lon = LairLon::parse(value.lon)?;
        let lat = LairLat::parse(value.lat)?;
        Ok(Self {
            title,
            description,
            image,
            lon,
            lat,
        })
    }
}

#[derive(thiserror::Error)]
pub enum LairInsertionError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LairInsertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for LairInsertionError {
    fn status_code(&self) -> StatusCode {
        match self {
            LairInsertionError::ValidationError(_) => StatusCode::BAD_REQUEST,
            LairInsertionError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(pool, request, lair_info)
)]
pub async fn insert_lair(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    lair_info: web::Json<LairInfo>,
) -> Result<HttpResponse, PublishError> {
    let credentials = basic_authentication(request.headers()).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let room_id = Uuid::new_v4();
    insert_lair_into_db(lair_info, &mut transaction, user_id, room_id)
        .await
        .expect("Couldn't push lair info into DB");
    Ok(HttpResponse::Ok().finish())
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
    transaction.execute(query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                response
                    .headers_mut()
                    .insert(actix_web::http::header::WWW_AUTHENTICATE, header_value);
                response
            }
        }
    }
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    // The header value, if present, must be a valid UTF8 string
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;
    let base64encoded_credentials = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'.")?;
    let decoded_credentials = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_credentials)
        .context("Failed to base64-decode 'Basic' credentials.")?;
    let decoded_credentials = String::from_utf8(decoded_credentials)
        .context("The decoded credential string is valid UTF8.")?;

    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?
        .to_string();
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}
