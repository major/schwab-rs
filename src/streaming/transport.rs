//! WebSocket transport abstraction for streaming.

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
