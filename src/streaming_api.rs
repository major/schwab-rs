//! `Client::stream()` entry point for WebSocket streaming sessions.

use tracing::instrument;

use crate::streaming::transport::{TungsteniteTransport, WsTransport};
use crate::streaming::{SessionCredentials, StreamingSession};
use crate::{Client, Error};

impl Client {
    /// Open a WebSocket streaming session for real-time level-one market data.
    ///
    /// Fetches the user preference to obtain streamer connection details,
    /// establishes a WebSocket connection, logs in, and returns a
    /// [`StreamingSession`] ready to accept subscriptions.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::AuthRequired`] if no bearer token is configured.
    /// Returns [`crate::Error::MissingRequiredParameter`] if user preferences
    /// do not contain the required streamer connection fields.
    /// Returns [`crate::Error::WebSocket`] if the WebSocket connection fails.
    /// Returns [`crate::Error::StreamLogin`] if the server rejects the login.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config};
    /// use schwab::EquityField;
    ///
    /// let config = Config::new().bearer_token("my_token");
    /// let client = Client::new(config);
    ///
    /// let session = client.stream().await?;
    /// let mut events = session.subscribe();
    /// session.subscribe_equities(&["AAPL"], &[EquityField::BidPrice, EquityField::AskPrice]).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn stream(&self) -> crate::Result<StreamingSession> {
        let bearer_token = self
            .config
            .bearer_token
            .clone()
            .ok_or(Error::AuthRequired)?;

        let prefs = self.get_user_preference().await?;
        let pref = prefs
            .into_iter()
            .next()
            .ok_or_else(|| Error::MissingRequiredParameter("no user preferences returned"))?;
        let streamer_info_vec = pref.streamer_info.ok_or_else(|| {
            Error::MissingRequiredParameter("streamer_info missing from user preference")
        })?;
        let info = streamer_info_vec
            .into_iter()
            .next()
            .ok_or_else(|| Error::MissingRequiredParameter("streamer_info list is empty"))?;
        let customer_id = info.schwab_client_customer_id.ok_or_else(|| {
            Error::MissingRequiredParameter("schwab_client_customer_id missing from streamer_info")
        })?;
        let correl_id = info.schwab_client_correl_id.ok_or_else(|| {
            Error::MissingRequiredParameter("schwab_client_correl_id missing from streamer_info")
        })?;
        let channel = info.schwab_client_channel.ok_or_else(|| {
            Error::MissingRequiredParameter("schwab_client_channel missing from streamer_info")
        })?;
        let function_id = info.schwab_client_function_id.ok_or_else(|| {
            Error::MissingRequiredParameter("schwab_client_function_id missing from streamer_info")
        })?;
        let socket_url = info.streamer_socket_url.ok_or_else(|| {
            Error::MissingRequiredParameter("streamer_socket_url missing from streamer_info")
        })?;

        let credentials = SessionCredentials {
            customer_id,
            correl_id,
            channel,
            function_id,
            bearer_token,
            socket_url: socket_url.clone(),
        };

        let transport = TungsteniteTransport::connect(&socket_url).await?;
        StreamingSession::new(transport, credentials).await
    }
}
