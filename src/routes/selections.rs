use actix_web::{web, HttpResponse};
use sqlx::{postgres::PgArguments, query::Query, Executor, PgPool, Postgres};
use uuid::Uuid;


pub async fn research_lair(pool: web::Data<PgPool>, room_id: Uuid) -> HttpResponse {
    match sqlx::query!(
        r#"
        SELECT * from rooms WHERE room_id = $1
            "#,
        room_id
    )
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    } 

}
