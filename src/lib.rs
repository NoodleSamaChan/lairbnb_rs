use std::net::TcpListener;
use serde::{Deserialize, Serialize};
use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

#[derive(Deserialize)]
struct FormData {
    email: String,
    name: String,
    password: String

}


async fn register(_form: web::Json<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new( || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/registrations", web::post().to(register))
    })
    .listen(listener)?
    .run();

    Ok(server)
}