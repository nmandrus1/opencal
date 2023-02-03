use actix_web::{dev::Server, web, App, Error, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

// basic health check end_point
async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

// entry point to the webscoket connection
async fn calendar_route() -> Result<HttpResponse, Error> {
    // start the web socket server here
    todo!()
}

// return an instance of our server
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}
