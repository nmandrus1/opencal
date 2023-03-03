use futures_util::{SinkExt, StreamExt};
use std::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

// ws test sends a ping with some bytes and should always return a pong with identical bytes
#[actix_rt::test]
async fn ws_endroute_works() {
    let addr = spawn_app();

    println!("Connecting to: {}", addr);

    // connect to websocket server
    let (mut ws_stream, _) = connect_async(&format!("ws://{}/ws", &addr))
        .await
        .expect("Failed to connect...");

    println!("Websocket handshake completed");

    let ping_msg = "michael is stinky".as_bytes().to_vec();

    // send a ping message to server
    ws_stream
        .send(Message::Ping(ping_msg.clone()))
        .await
        .expect("Message to WS failed to send...");

    ws_stream.flush().await.expect("Failed to flush sink");

    // we should immediately recieve a pong message from the server
    // with the same contents as the ping we just sent
    while let Some(message) = ws_stream.next().await {
        match message.unwrap() {
            Message::Pong(m) => {
                println!("Pong Message: {}", String::from_utf8_lossy(&m));
                assert_eq!(m, ping_msg);
                break;
            }
            _ => unreachable!(),
        }
    }
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
