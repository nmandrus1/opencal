use std::net::TcpListener;

use opencal::run;

// hello to all reading this, I am currently daf and vibing super hard with sebas ╰⋃╯

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // enter the "main" function for our server
    // have any errors "bubble up" to the binary entry point

    let listenser = TcpListener::bind("127.0.0.1:8000")?;

    run(listenser)?.await
}
