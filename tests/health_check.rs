use reqwest;
use std::net::TcpListener;

// health check should always return 200 with no body
#[actix_rt::test]
async fn health_check_works() {
    let addr = spawn_app();

    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/health_check", &addr))
        .send()
        .await
        .expect("Failed to send request to server");

    // check response
    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());
}

fn spawn_app() -> String {
    // use port 0 to make the OS pick a random port that isnt being used
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to address");
    let port = listener.local_addr().unwrap().port();

    let server = opencal::run(listener).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    // return address of server
    format!("127.0.0.1:{}", port)
}
