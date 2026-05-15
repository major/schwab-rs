//! WebSocket transport abstraction for streaming.

use futures_util::{SinkExt, StreamExt};
use std::future::Future;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

/// Abstraction over a WebSocket connection for testing and production.
///
/// Implementors handle the raw WebSocket wire protocol; the streaming
/// session handles reconnect logic and message dispatch.
#[allow(missing_docs)]
pub(crate) trait WsTransport: Sized + Send {
    /// Establish a new WebSocket connection to `url`.
    fn connect(url: &str) -> impl Future<Output = crate::Result<Self>> + Send;
    /// Send a text message over the WebSocket.
    fn send(&mut self, msg: String) -> impl Future<Output = crate::Result<()>> + Send;
    /// Receive the next text message. Returns `None` when the connection closes.
    fn next(&mut self) -> impl Future<Output = crate::Result<Option<String>>> + Send;
    /// Close the WebSocket connection gracefully.
    fn close(&mut self) -> impl Future<Output = crate::Result<()>> + Send;
}

/// A real WebSocket transport backed by tokio-tungstenite.
///
/// This is the production implementation of [`WsTransport`].
/// Test code can substitute a mock implementation of the trait.
#[allow(missing_docs)]
pub(crate) struct TungsteniteTransport {
    ws: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
}

impl WsTransport for TungsteniteTransport {
    async fn connect(url: &str) -> crate::Result<Self> {
        let (ws, _response) = connect_async(url)
            .await
            .map_err(|error| crate::Error::WebSocket(Box::new(error)))?;
        Ok(Self { ws })
    }

    async fn send(&mut self, msg: String) -> crate::Result<()> {
        self.ws
            .send(Message::Text(msg.into()))
            .await
            .map_err(|error| crate::Error::WebSocket(Box::new(error)))
    }

    async fn next(&mut self) -> crate::Result<Option<String>> {
        loop {
            match self.ws.next().await {
                Some(Ok(Message::Text(text))) => return Ok(Some(text.to_string())),
                Some(Ok(Message::Close(_))) | None => return Ok(None),
                Some(Ok(_)) => continue, // Ignore Ping, Pong, Binary
                Some(Err(error)) => return Err(crate::Error::WebSocket(Box::new(error))),
            }
        }
    }

    async fn close(&mut self) -> crate::Result<()> {
        self.ws
            .close(None)
            .await
            .map_err(|error| crate::Error::WebSocket(Box::new(error)))
    }
}
