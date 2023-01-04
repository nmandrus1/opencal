use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use calcium::{Event, EventCalendar, EventError};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

struct AppState {
    cal: Mutex<EventCalendar>,
}

#[derive(Deserialize)]
struct RangeQuery {
    start: NaiveDateTime,
    end: NaiveDateTime,
}

#[get("/event_range")]
async fn event_range(query: web::Query<RangeQuery>, data: web::Data<AppState>) -> impl Responder {
    let cal = data.cal.lock().unwrap();
    let events: Vec<&Event> = cal
        .events_in_range(query.start, query.end)
        .map(|(_, evt)| evt)
        .collect();

    serde_json::to_string(&events).unwrap()
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
    let date = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2023, 7, 1).unwrap(),
        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    );

    println!("Serialized Date: {}", serde_json::to_string(&date).unwrap());

    let app_state = AppState {
        cal: Mutex::new(EventCalendar::default()),
    };

    app_state.cal.lock().unwrap().add_event(Event::new(
        "Classes Begin".into(),
        &NaiveDate::from_ymd_opt(2023, 1, 9).unwrap(),
    ));

    app_state.cal.lock().unwrap().add_event(Event::new(
        "First Council".into(),
        &NaiveDate::from_ymd_opt(2023, 1, 8).unwrap(),
    ));

    app_state.cal.lock().unwrap().add_event(Event::new(
        "Funnt Event".into(),
        &NaiveDate::from_ymd_opt(2023, 1, 10).unwrap(),
    ));

    let app_state = web::Data::new(app_state);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(event_range)
            .service(first_event)
            .service(hello)
            .service(echo)
            .service(web::scope("/app").service(index))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
