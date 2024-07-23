use std::future::{ready, Ready};

use actix_web::{web, FromRequest, HttpRequest, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormDataLair {
    title: String,
    image: String,
    description: String,
    lon: f64,
    lat: f64,
}

pub async fn insert_lair(account_id: Uuid, lair_info: FormDataLair, pool: web::Data<PgPool>) -> HttpResponse  {
    match sqlx::query!(
        r#"
    INSERT INTO rooms (id, title, image, description, lon, lat, room_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        account_id,
        lair_info.title,
        lair_info.image,
        lair_info.description,
        lair_info.lon,
        lair_info.lat,
        Uuid::new_v4(),
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}