use actix_web::{
    get,
    http::{header::ContentType, StatusCode},
    post, web, App, HttpResponse, HttpServer, Responder, ResponseError,
};

use calib::{Event, EventCalendar};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::Deserialize;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
enum ServerError {
    #[error("No event for corresponding uuid")]
    EventNotFound,
}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::EventNotFound => StatusCode::NOT_FOUND,
        }
    }
}

struct AppState {
    cal: Mutex<EventCalendar>,
}

#[derive(Deserialize)]
struct RangeQuery {
    start: NaiveDateTime,
    end: NaiveDateTime,
}

#[derive(Deserialize)]
struct EventQuery {
    uuid: String,
}

#[get("/event")]
async fn event(
    query: web::Query<EventQuery>,
    data: web::Data<AppState>,
) -> Result<String, ServerError> {
    let cal = data.cal.lock().unwrap();
    println!("event request: {}", query.uuid);
    match cal.get(query.uuid.as_str()) {
        Some(evt) => Ok(evt.serialize()),
        None => Err(ServerError::EventNotFound),
    }
}

#[get("/remove_event")]
async fn remove_event(
    query: web::Query<EventQuery>,
    data: web::Data<AppState>,
) -> Result<String, ServerError> {
    let mut cal = data.cal.lock().unwrap();
    match cal.remove(query.uuid.as_str()) {
        Some(e) => Ok(format!("Event: {} removed...", e.name())),
        None => Err(ServerError::EventNotFound),
    }
}

#[get("/event_range")]
async fn event_range(query: web::Query<RangeQuery>, data: web::Data<AppState>) -> impl Responder {
    let cal = data.cal.lock().unwrap();
    let events: Vec<String> = cal
        .events_in_range(query.start, query.end)
        .map(|evt| evt.serialize())
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
            .service(event)
            .service(remove_event)
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
