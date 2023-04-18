// use env_logger::Env;
use std::net::TcpListener;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use opencal::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // enter the "main" function for our server
    // have any errors "bubble up" to the binary entry point
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    LogTracer::init().expect("Failed to set logger");

    // Filters out spans based on their log level and origins
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Subscriber layers allow for processing pipelines for span data
    let subscriber = Registry::default().with(env_filter);
    set_global_default(subscriber).expect("Failed to set subscriber");

    let listener = TcpListener::bind("127.0.0.1:8000")?;

    run(listener)?.await
}
