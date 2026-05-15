//! WebSocket transport abstraction for streaming.

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// Abstraction over a WebSocket connection for testing and production.
///
/// Implementors handle the raw WebSocket wire protocol; the streaming
/// session handles reconnect logic and message dispatch.
#[allow(missing_docs)]
pub(crate) trait WsTransport: Sized + Send {
    /// Establish a new WebSocket connection to `url`.
    async fn connect(url: &str) -> crate::Result<Self>;
    /// Send a text message over the WebSocket.
    async fn send(&mut self, msg: String) -> crate::Result<()>;
    /// Receive the next text message. Returns `None` when the connection closes.
    async fn next(&mut self) -> crate::Result<Option<String>>;
    /// Close the WebSocket connection gracefully.
    async fn close(&mut self) -> crate::Result<()>;
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
            .map_err(crate::Error::WebSocket)?;
        Ok(Self { ws })
    }

    async fn send(&mut self, msg: String) -> crate::Result<()> {
        self.ws
            .send(Message::Text(msg.into()))
            .await
            .map_err(crate::Error::WebSocket)
    }

    async fn next(&mut self) -> crate::Result<Option<String>> {
        loop {
            match self.ws.next().await {
                Some(Ok(Message::Text(text))) => return Ok(Some(text.to_string())),
                Some(Ok(Message::Close(_))) | None => return Ok(None),
                Some(Ok(_)) => continue, // Ignore Ping, Pong, Binary
                Some(Err(e)) => return Err(crate::Error::WebSocket(e)),
            }
        }
    }

    async fn close(&mut self) -> crate::Result<()> {
        self.ws
            .close(None)
            .await
            .map_err(crate::Error::WebSocket)
    }
}
