use std::sync::Arc;

use error::ReceiveError;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use pianoverse_proto::client_message::EventType;
use prost::Message;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    tungstenite::{self},
    MaybeTlsStream, WebSocketStream,
};
mod error;

const WS_URL: &str = "wss://pianoverse.net/";

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone)]
pub struct Client {
    write: Arc<Mutex<SplitSink<WebSocket, tungstenite::Message>>>,
    read: Arc<Mutex<SplitStream<WebSocket>>>,
}

impl Client {
    pub async fn connect() -> Result<Client, tungstenite::Error> {
        let (conn, _) = tokio_tungstenite::connect_async(WS_URL).await?;
        let (write, read) = conn.split();
        Ok(Client {
            write: Arc::new(Mutex::new(write)),
            read: Arc::new(Mutex::new(read)),
        })
    }

    pub async fn send(
        &self,
        msg: pianoverse_proto::ClientMessage,
    ) -> Result<(), tungstenite::Error> {
        self.write
            .lock()
            .await
            .send(tungstenite::Message::Binary(msg.encode_to_vec()))
            .await
    }

    pub async fn recv(&self) -> Result<Option<pianoverse_proto::ServerMessage>, ReceiveError> {
        let msg = self.read.lock().await.next().await;
        if msg.is_none() {
            return Ok(None);
        }

        let msg = match msg.unwrap()? {
            tungstenite::Message::Binary(b) => {
                #[cfg(debug_assertions)]
                eprintln!("received: {:?}", hex::encode(&b));
                b
            }
            _ => return Err(ReceiveError::UnexpectedMessage),
        };
        Ok(Some(pianoverse_proto::ServerMessage::decode(&msg[..])?))
    }

    pub async fn join_or_create_room<N: Into<String>>(
        &self,
        name: N,
        private: bool,
    ) -> Result<(), tungstenite::Error> {
        self.send(pianoverse_proto::ClientMessage {
            event: EventType::Room.into(),
            room: Some(pianoverse_proto::client_message::Room {
                room: name.into(),
                private,
            }),
            ..Default::default()
        })
        .await
    }

    // async fn update_profile(
    //     &self,
    //     profile: pianoverse_proto::Profile,
    // ) -> Result<(), tungstenite::Error> {
    //     self.send(pianoverse_proto::ClientMessage {
    //         event: EventType::Profile.into(),
    //         profile: Some(profile),
    //         ..Default::default()
    //     })
    //     .await
    // }

    pub async fn send_chat_message<T: Into<String>>(
        &self,
        text: T,
    ) -> Result<(), tungstenite::Error> {
        self.send(pianoverse_proto::ClientMessage {
            event: EventType::Chat.into(),
            chat: text.into(),
            ..Default::default()
        })
        .await
    }

    pub async fn press(&self, key: u32, vel: u32) -> Result<(), tungstenite::Error> {
        self.send(pianoverse_proto::ClientMessage {
            event: EventType::Press.into(),
            press: Some(pianoverse_proto::Press {
                key,
                vel,
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
    }

    pub async fn release(&self, key: u32) -> Result<(), tungstenite::Error> {
        self.send(pianoverse_proto::ClientMessage {
            event: EventType::Release.into(),
            release: Some(pianoverse_proto::Release {
                key,
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
    }
}
