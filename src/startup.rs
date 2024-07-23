use std::net::TcpListener;
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use sqlx::PgPool;
use crate::routes::{insert_lair, register, research_lair};

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new( move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/registrations", web::post().to(register))
            .route("/insertion", web::post().to(insert_lair))
            .route("/lair_research", web::get().to(research_lair))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}