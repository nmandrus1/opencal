use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use calcium_lib::{Event, EventCalendar, EventError};
use chrono::NaiveDate;
use std::sync::Mutex;

struct AppState {
    cal: Mutex<EventCalendar>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

#[get("/first")]
async fn first_event(data: web::Data<AppState>) -> impl Responder {
    let cal = data.cal.lock().unwrap();
    let evt = cal.first_event().unwrap();
    HttpResponse::Ok().body(format!("First Event is {} on {}", evt.name(), evt.start()))
}

#[get("/index.html")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Big Funny!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hi there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = AppState {
        cal: Mutex::new(EventCalendar::default()),
    };

    app_state.cal.lock().unwrap().add_event(Event::new(
        "Niels Birthday".into(),
        &NaiveDate::from_ymd_opt(2023, 7, 12).unwrap(),
    ));

    let app_state = web::Data::new(app_state);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(first_event)
            .service(hello)
            .service(echo)
            .service(web::scope("/app").service(index))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
