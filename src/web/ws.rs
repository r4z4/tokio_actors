use futures_util::{stream::SplitSink, SinkExt as _, StreamExt as _};
use tokio::io::{self, AsyncBufReadExt, AsyncRead, AsyncWrite};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};

pub async fn start() {
    let url = "wss://echo.websocket.events";
    println!("Connecting to - {}", url);
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to Network");

    let (mut write, mut read) = ws_stream.split();

    if let Some(message) = read.next().await {
        let message = message.expect("Failed to read msg");
        println!("Received a message - {}", message);
    }

    let msg = Message::Text("aloha echo server".into());
    println!("Sending message - {}", msg);
    write.send(msg).await.expect("Failed to send message");

    if let Some(message) = read.next().await {
        let message = message.expect("Failed to read msg");
        println!("Received a message - {}", message);
    }
}

pub async fn read_and_send_messages(
    mut write: SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
) {
    let mut reader = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = reader.next_line().await.expect("Failed to read") {
        if !line.trim().is_empty() {
            write
                .send(Message::Text(line))
                .await
                .expect("Filed to send")
        }
    }
}
