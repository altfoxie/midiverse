use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(pianoverse_client::Client::connect().await.unwrap());

    let client_clone = client.clone();
    tokio::spawn(async move {
        loop {
            let _msg = client_clone.recv().await.unwrap();
        }
    });

    client.join_or_create_room("Lobby", false).await.unwrap();
    client.send_chat_message("Test!").await.unwrap();

    loop {}
}
