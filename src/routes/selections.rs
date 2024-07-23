use actix_web::{web, HttpResponse};
use sqlx::{PgPool, Executor};
use uuid::Uuid;

pub async fn research_lair(pool: web::Data<PgPool>, room_id: Uuid) -> HttpResponse {
    /*
    match sqlx::query!(
        r#"
        SELECT * from rooms WHERE room_id = $1
            "#,
        room_id
    )
    .execute(pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    } */
/*
    let account = sqlx::query!(r#"
        SELECT * from rooms WHERE room_id = $1
            "#,
        room_id
    )
    .fetch_all(&mut pool)
    .await?;*/

    HttpResponse::Ok().finish() 

}