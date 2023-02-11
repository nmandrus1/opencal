use actix::{Actor, Addr, StreamHandler};
use actix_web::{dev::Server, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use session::WsCalSession;
use std::{net::TcpListener, time::Instant};

mod server;
mod session;

// basic health check end_point
async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

// entry point to the webscoket connection
async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::CalServer>>,
) -> Result<HttpResponse, Error> {
    // start the web socket server here
    println!("Knock Knock");
    ws::start(
        session::WsCalSession {
            id: 0,
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

// return an instance of our server
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    // start calendar server
    let server = server::CalServer::new().start();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .route("/health_check", web::get().to(health_check))
            .route("/ws", web::get().to(ws_route))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
