use actix::{Actor, Addr};
use actix_web::middleware::Logger;
use actix_web::{dev::Server, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use std::io::Write;
use std::{net::TcpListener, time::Instant};
use uuid::Uuid;

mod calendar;
mod event;
mod server;
mod session;

// basic health check end_point
async fn health_check() -> impl Responder {
    let requestid = Uuid::new_v4();
    tracing::info!(
        "Request_id: {} made to the health_check endroute",
        requestid
    );
    HttpResponse::Ok().finish()
}

// entry point to the websocket connection
async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::CalServer>>,
) -> Result<HttpResponse, Error> {
    // start the web socket server here
    let requestid = Uuid::new_v4();
    tracing::info!("Request_id: {} made to the websocket endroute", requestid);

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
    let requestid = Uuid::new_v4();
    tracing::info!("Request_id: {} - New server created", requestid);

    // start calendar server
    let server = server::CalServer::new().start();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(server.clone()))
            .route("/health_check", web::get().to(health_check))
            .route("/ws", web::get().to(ws_route))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
