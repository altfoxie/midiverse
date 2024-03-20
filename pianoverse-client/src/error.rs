#[derive(thiserror::Error, Debug)]
pub enum ReceiveError {
    #[error("WebSocket read error: {0}")]
    Read(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Decode error: {0}")]
    Prost(#[from] prost::DecodeError),

    #[error("Unexpected message type")]
    UnexpectedMessage,
}
