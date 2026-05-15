//! WebSocket streaming client for Schwab real-time market data.

use std::sync::Arc;

use tokio::sync::{Mutex, broadcast, mpsc, oneshot};

use crate::models::streaming::{
    LevelOneEquity, LevelOneForex, LevelOneFutures, LevelOneFuturesOption, LevelOneOption,
    StreamData, StreamEvent, StreamResponse, equities::EquityField, forex::ForexField,
    futures::FuturesField, futures_options::FuturesOptionField, options::OptionField,
};
use crate::streaming::protocol::ParsedMessage;
use crate::streaming::transport::WsTransport;

pub(crate) mod protocol;
pub(crate) mod transport;

type CommandAck = oneshot::Sender<crate::Result<()>>;
type LogoutAck = oneshot::Sender<crate::Result<()>>;

enum SessionCommand {
    Send { text: String, ack: CommandAck },
}

/// Streaming WebSocket session for Schwab level-one market data.
///
/// Create a session through `Client::stream()` once that entry point is
/// available. Each session owns a background WebSocket task, broadcasts parsed
/// [`StreamEvent`] values, and replays active subscriptions after reconnecting.
pub struct StreamingSession {
    cmd_tx: mpsc::Sender<SessionCommand>,
    logout_tx: mpsc::Sender<LogoutAck>,
    event_tx: broadcast::Sender<StreamEvent>,
    subs: Arc<Mutex<Vec<StoredSub>>>,
    credentials: Arc<SessionCredentials>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredSub {
    service: String,
    symbols: Vec<String>,
    field_indices: Vec<u32>,
}

/// Connection details required for a streaming LOGIN command.
pub(crate) struct SessionCredentials {
    pub(crate) customer_id: String,
    pub(crate) correl_id: String,
    pub(crate) channel: String,
    pub(crate) function_id: String,
    pub(crate) bearer_token: String,
    pub(crate) socket_url: String,
}

impl StreamingSession {
    /// Create a streaming session from an already-connected transport.
    pub(crate) async fn new<T: WsTransport + 'static>(
        mut transport: T,
        credentials: SessionCredentials,
    ) -> crate::Result<Self> {
        login(&mut transport, &credentials).await?;

        let (event_tx, _) = broadcast::channel(1024);
        let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>(64);
        let (logout_tx, logout_rx) = mpsc::channel::<LogoutAck>(1);
        let subs = Arc::new(Mutex::new(Vec::new()));
        let credentials = Arc::new(credentials);

        tokio::spawn(run_message_loop::<T>(
            transport,
            cmd_rx,
            logout_rx,
            event_tx.clone(),
            subs.clone(),
            credentials.clone(),
        ));

        Ok(Self {
            cmd_tx,
            logout_tx,
            event_tx,
            subs,
            credentials,
        })
    }

    /// Subscribe to streaming events from this session.
    ///
    /// Returns a [`tokio::sync::broadcast::Receiver`] that delivers
    /// [`StreamEvent`] values. Multiple callers may subscribe; each gets an
    /// independent receiver.
    ///
    /// The broadcast channel has a buffer of 1024 events. If a receiver falls
    /// behind, it receives [`tokio::sync::broadcast::error::RecvError::Lagged`].
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<StreamEvent> {
        self.event_tx.subscribe()
    }

    /// Disconnect from the streaming service.
    ///
    /// Sends a LOGOUT command, closes the WebSocket transport, and stops the
    /// background message loop.
    ///
    /// # Errors
    /// Returns [`crate::Error::StreamProtocol`] if the session task has already
    /// stopped or if the LOGOUT command cannot be delivered by the transport.
    #[tracing::instrument(skip_all)]
    pub async fn disconnect(&self) -> crate::Result<()> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.logout_tx
            .send(ack_tx)
            .await
            .map_err(|e| crate::Error::StreamProtocol(e.to_string()))?;

        ack_rx
            .await
            .map_err(|e| crate::Error::StreamProtocol(e.to_string()))?
    }

    /// Subscribe to level-one equity data for the given symbols.
    ///
    /// # Errors
    /// Returns [`crate::Error::EmptySymbols`] if `symbols` is empty.
    /// Returns [`crate::Error::StreamProtocol`] if no fields are provided, the
    /// command cannot be serialized, or the session command loop is stopped.
    #[tracing::instrument(skip_all)]
    pub async fn subscribe_equities(
        &self,
        symbols: &[&str],
        fields: &[EquityField],
    ) -> crate::Result<()> {
        let field_indices = fields.iter().map(EquityField::index).collect::<Vec<_>>();
        self.subscribe_service("2", "LEVELONE_EQUITIES", symbols, field_indices)
            .await
    }

    /// Subscribe to level-one option data for the given symbols.
    ///
    /// # Errors
    /// Returns [`crate::Error::EmptySymbols`] if `symbols` is empty.
    /// Returns [`crate::Error::StreamProtocol`] if no fields are provided, the
    /// command cannot be serialized, or the session command loop is stopped.
    #[tracing::instrument(skip_all)]
    pub async fn subscribe_options(
        &self,
        symbols: &[&str],
        fields: &[OptionField],
    ) -> crate::Result<()> {
        let field_indices = fields.iter().map(OptionField::index).collect::<Vec<_>>();
        self.subscribe_service("3", "LEVELONE_OPTIONS", symbols, field_indices)
            .await
    }

    /// Subscribe to level-one futures data for the given symbols.
    ///
    /// # Errors
    /// Returns [`crate::Error::EmptySymbols`] if `symbols` is empty.
    /// Returns [`crate::Error::StreamProtocol`] if no fields are provided, the
    /// command cannot be serialized, or the session command loop is stopped.
    #[tracing::instrument(skip_all)]
    pub async fn subscribe_futures(
        &self,
        symbols: &[&str],
        fields: &[FuturesField],
    ) -> crate::Result<()> {
        let field_indices = fields.iter().map(FuturesField::index).collect::<Vec<_>>();
        self.subscribe_service("4", "LEVELONE_FUTURES", symbols, field_indices)
            .await
    }

    /// Subscribe to level-one futures option data for the given symbols.
    ///
    /// # Errors
    /// Returns [`crate::Error::EmptySymbols`] if `symbols` is empty.
    /// Returns [`crate::Error::StreamProtocol`] if no fields are provided, the
    /// command cannot be serialized, or the session command loop is stopped.
    #[tracing::instrument(skip_all)]
    pub async fn subscribe_futures_options(
        &self,
        symbols: &[&str],
        fields: &[FuturesOptionField],
    ) -> crate::Result<()> {
        let field_indices = fields
            .iter()
            .map(FuturesOptionField::index)
            .collect::<Vec<_>>();
        self.subscribe_service("5", "LEVELONE_FUTURES_OPTIONS", symbols, field_indices)
            .await
    }

    /// Subscribe to level-one forex data for the given symbols.
    ///
    /// # Errors
    /// Returns [`crate::Error::EmptySymbols`] if `symbols` is empty.
    /// Returns [`crate::Error::StreamProtocol`] if no fields are provided, the
    /// command cannot be serialized, or the session command loop is stopped.
    #[tracing::instrument(skip_all)]
    pub async fn subscribe_forex(
        &self,
        symbols: &[&str],
        fields: &[ForexField],
    ) -> crate::Result<()> {
        let field_indices = fields.iter().map(ForexField::index).collect::<Vec<_>>();
        self.subscribe_service("6", "LEVELONE_FOREX", symbols, field_indices)
            .await
    }

    async fn subscribe_service(
        &self,
        request_id: &str,
        service: &str,
        symbols: &[&str],
        field_indices: Vec<u32>,
    ) -> crate::Result<()> {
        if symbols.is_empty() {
            return Err(crate::Error::EmptySymbols);
        }
        if field_indices.is_empty() {
            return Err(crate::Error::StreamProtocol(
                "at least one streaming field is required".to_string(),
            ));
        }

        let message = protocol::build_subs(
            request_id,
            service,
            &self.credentials.customer_id,
            &self.credentials.correl_id,
            symbols,
            &field_indices,
        )?;
        let (ack_tx, ack_rx) = oneshot::channel();
        self.cmd_tx
            .send(SessionCommand::Send {
                text: message,
                ack: ack_tx,
            })
            .await
            .map_err(|e| crate::Error::StreamProtocol(e.to_string()))?;
        ack_rx
            .await
            .map_err(|e| crate::Error::StreamProtocol(e.to_string()))??;

        let mut subs = self.subs.lock().await;
        subs.retain(|sub| sub.service != service);
        subs.push(StoredSub {
            service: service.to_string(),
            symbols: symbols.iter().map(|symbol| (*symbol).to_string()).collect(),
            field_indices,
        });

        Ok(())
    }
}

async fn login<T: WsTransport>(
    transport: &mut T,
    credentials: &SessionCredentials,
) -> crate::Result<()> {
    let login_msg = protocol::build_login(
        &credentials.customer_id,
        &credentials.correl_id,
        &credentials.channel,
        &credentials.function_id,
        &credentials.bearer_token,
    )?;
    transport.send(login_msg).await?;

    let response_text = transport.next().await?.ok_or_else(|| {
        crate::Error::StreamProtocol("connection closed before login response".to_string())
    })?;

    match check_login_response(&response_text) {
        LoginResult::Ok => Ok(()),
        LoginResult::Denied(code, message) => Err(crate::Error::StreamLogin { code, message }),
        LoginResult::Error => Err(crate::Error::StreamProtocol(
            "streaming login response did not contain a success code".to_string(),
        )),
    }
}

async fn run_message_loop<T: WsTransport + 'static>(
    mut transport: T,
    mut cmd_rx: mpsc::Receiver<SessionCommand>,
    mut logout_rx: mpsc::Receiver<LogoutAck>,
    event_tx: broadcast::Sender<StreamEvent>,
    subs: Arc<Mutex<Vec<StoredSub>>>,
    credentials: Arc<SessionCredentials>,
) {
    loop {
        tokio::select! {
            biased;
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(SessionCommand::Send { text, ack }) => {
                        if let Err(error) = transport.send(text).await {
                            let error_text = error.to_string();
                            tracing::warn!("failed to send streaming command: {error_text}");
                            let _ = ack.send(Err(error));
                            let _ = event_tx.send(StreamEvent::Disconnected {
                                error: Some(error_text),
                            });
                            match wait_for_reconnect::<T>(&event_tx, &subs, &credentials, &mut logout_rx).await {
                                ReconnectOutcome::Connected(new_transport) => transport = new_transport,
                                ReconnectOutcome::Stopped => break,
                            }
                        } else {
                            let _ = ack.send(Ok(()));
                        }
                    }
                    None => {
                        let _ = transport.close().await;
                        break;
                    }
                }
            }
            msg_result = transport.next() => {
                match msg_result {
                    Ok(Some(text)) => match dispatch_message(&text, &event_tx) {
                        DispatchAction::Continue => {}
                        DispatchAction::Reconnect(error) => {
                            let _ = event_tx.send(StreamEvent::Disconnected { error: Some(error) });
                            match wait_for_reconnect::<T>(&event_tx, &subs, &credentials, &mut logout_rx).await {
                                ReconnectOutcome::Connected(new_transport) => transport = new_transport,
                                ReconnectOutcome::Stopped => break,
                            }
                        }
                        DispatchAction::Stop(error) => {
                            let _ = event_tx.send(StreamEvent::Disconnected { error: Some(error) });
                            let _ = transport.close().await;
                            break;
                        }
                    },
                    Ok(None) => {
                        let _ = event_tx.send(StreamEvent::Disconnected { error: None });
                        match wait_for_reconnect::<T>(&event_tx, &subs, &credentials, &mut logout_rx).await {
                            ReconnectOutcome::Connected(new_transport) => transport = new_transport,
                            ReconnectOutcome::Stopped => break,
                        }
                    }
                    Err(error) => {
                        let _ = event_tx.send(StreamEvent::Disconnected {
                            error: Some(error.to_string()),
                        });
                        match wait_for_reconnect::<T>(&event_tx, &subs, &credentials, &mut logout_rx).await {
                            ReconnectOutcome::Connected(new_transport) => transport = new_transport,
                            ReconnectOutcome::Stopped => break,
                        }
                    }
                }
            }
            logout = logout_rx.recv() => {
                match logout {
                    Some(ack) => {
                        let result = send_logout(&mut transport, &credentials).await;
                        let _ = ack.send(result);
                        break;
                    }
                    None => {
                        let _ = transport.close().await;
                        break;
                    }
                }
            }
        }
    }
}

async fn send_logout<T: WsTransport>(
    transport: &mut T,
    credentials: &SessionCredentials,
) -> crate::Result<()> {
    let logout = protocol::build_logout(&credentials.customer_id, &credentials.correl_id)?;
    transport.send(logout).await?;
    transport.close().await
}

enum ReconnectOutcome<T> {
    Connected(T),
    Stopped,
}

async fn wait_for_reconnect<T: WsTransport>(
    event_tx: &broadcast::Sender<StreamEvent>,
    subs: &Arc<Mutex<Vec<StoredSub>>>,
    credentials: &SessionCredentials,
    logout_rx: &mut mpsc::Receiver<LogoutAck>,
) -> ReconnectOutcome<T> {
    let reconnect = reconnect::<T>(credentials, event_tx, subs);
    tokio::pin!(reconnect);

    tokio::select! {
        new_transport = &mut reconnect => {
            match new_transport {
                Some(transport) => ReconnectOutcome::Connected(transport),
                None => ReconnectOutcome::Stopped,
            }
        }
        logout = logout_rx.recv() => {
            if let Some(ack) = logout {
                let _ = ack.send(Err(crate::Error::StreamProtocol(
                    "cannot send LOGOUT while reconnecting".to_string(),
                )));
            }
            ReconnectOutcome::Stopped
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DispatchAction {
    Continue,
    Reconnect(String),
    Stop(String),
}

fn dispatch_message(text: &str, event_tx: &broadcast::Sender<StreamEvent>) -> DispatchAction {
    let messages = match protocol::parse_message(text) {
        Ok(messages) => messages,
        Err(error) => {
            tracing::warn!("failed to parse streaming message: {error}");
            return DispatchAction::Continue;
        }
    };

    let mut action = DispatchAction::Continue;
    for message in messages {
        match message {
            ParsedMessage::Heartbeat(timestamp) => {
                let _ = event_tx.send(StreamEvent::Heartbeat(timestamp));
            }
            ParsedMessage::Response(response) => {
                let code = response.content.as_ref().and_then(|content| content.code);
                let response_event = StreamResponse {
                    service: response.service,
                    command: response.command,
                    request_id: response.requestid,
                    code,
                    message: response
                        .content
                        .as_ref()
                        .and_then(|content| content.msg.clone()),
                };
                let _ = event_tx.send(StreamEvent::Response(response_event));

                match code {
                    Some(3) => {
                        action = DispatchAction::Stop("LOGIN_DENIED (code=3)".to_string());
                    }
                    Some(12) if action == DispatchAction::Continue => {
                        action =
                            DispatchAction::Reconnect("CLOSE_CONNECTION (code=12)".to_string());
                    }
                    Some(30) if action == DispatchAction::Continue => {
                        action = DispatchAction::Reconnect("STOP_STREAMING (code=30)".to_string());
                    }
                    _ => {}
                }
            }
            ParsedMessage::Data(data_message) => {
                if let Some(data) = parse_data_message(data_message) {
                    let _ = event_tx.send(StreamEvent::Data(data));
                }
            }
        }
    }

    action
}

fn parse_data_message(data_message: protocol::StreamDataMessage) -> Option<StreamData> {
    let service = data_message.service.as_deref().unwrap_or("");
    let content = data_message.content?;

    match service {
        "LEVELONE_EQUITIES" => Some(StreamData::LevelOneEquities(parse_items(
            &content,
            LevelOneEquity::from_value,
        ))),
        "LEVELONE_OPTIONS" => Some(StreamData::LevelOneOptions(parse_items(
            &content,
            LevelOneOption::from_value,
        ))),
        "LEVELONE_FUTURES" => Some(StreamData::LevelOneFutures(parse_items(
            &content,
            LevelOneFutures::from_value,
        ))),
        "LEVELONE_FUTURES_OPTIONS" => Some(StreamData::LevelOneFuturesOptions(parse_items(
            &content,
            LevelOneFuturesOption::from_value,
        ))),
        "LEVELONE_FOREX" => Some(StreamData::LevelOneForex(parse_items(
            &content,
            LevelOneForex::from_value,
        ))),
        other => {
            tracing::warn!("unknown streaming service: {other}");
            None
        }
    }
}

fn parse_items<T>(
    content: &[serde_json::Map<String, serde_json::Value>],
    parse: fn(&serde_json::Value) -> Option<T>,
) -> Vec<T> {
    content
        .iter()
        .filter_map(|value| parse(&serde_json::Value::Object(value.clone())))
        .collect()
}

async fn reconnect<T: WsTransport>(
    credentials: &SessionCredentials,
    event_tx: &broadcast::Sender<StreamEvent>,
    subs: &Arc<Mutex<Vec<StoredSub>>>,
) -> Option<T> {
    let mut delay = tokio::time::Duration::from_secs(1);
    let max_delay = tokio::time::Duration::from_secs(30);

    for attempt in 1u32..=10 {
        let _ = event_tx.send(StreamEvent::Reconnecting { attempt });

        let jitter_ms = rand::random::<u64>() % 500;
        tokio::time::sleep(delay + tokio::time::Duration::from_millis(jitter_ms)).await;

        let mut transport = match T::connect(&credentials.socket_url).await {
            Ok(transport) => transport,
            Err(error) => {
                tracing::warn!("reconnect attempt {attempt} failed during connect: {error}");
                delay = (delay * 2).min(max_delay);
                continue;
            }
        };

        if let Err(error) = login(&mut transport, credentials).await {
            match error {
                crate::Error::StreamLogin { code: 3, message } => {
                    let _ = event_tx.send(StreamEvent::Disconnected {
                        error: Some(format!("LOGIN_DENIED (code=3): {message}")),
                    });
                    return None;
                }
                other => {
                    tracing::warn!("reconnect attempt {attempt} failed during login: {other}");
                    delay = (delay * 2).min(max_delay);
                    continue;
                }
            }
        }

        replay_subscriptions(&mut transport, credentials, subs).await;
        let _ = event_tx.send(StreamEvent::Reconnected);
        return Some(transport);
    }

    let _ = event_tx.send(StreamEvent::Disconnected {
        error: Some("max reconnect attempts exceeded".to_string()),
    });
    None
}

async fn replay_subscriptions<T: WsTransport>(
    transport: &mut T,
    credentials: &SessionCredentials,
    subs: &Arc<Mutex<Vec<StoredSub>>>,
) {
    let stored_subs = subs.lock().await.clone();
    for sub in stored_subs {
        let symbols = sub.symbols.iter().map(String::as_str).collect::<Vec<_>>();
        match protocol::build_subs(
            "1",
            &sub.service,
            &credentials.customer_id,
            &credentials.correl_id,
            &symbols,
            &sub.field_indices,
        ) {
            Ok(message) => {
                if let Err(error) = transport.send(message).await {
                    tracing::warn!("failed to replay streaming subscription: {error}");
                }
            }
            Err(error) => {
                tracing::warn!("failed to rebuild streaming subscription: {error}");
            }
        }
    }
}

enum LoginResult {
    Ok,
    Denied(u32, String),
    Error,
}

fn check_login_response(text: &str) -> LoginResult {
    let messages = match protocol::parse_message(text) {
        Ok(messages) => messages,
        Err(_) => return LoginResult::Error,
    };

    for message in messages {
        if let ParsedMessage::Response(response) = message
            && let Some(content) = response.content
        {
            let code = content.code.unwrap_or(0);
            if code == 0 {
                return LoginResult::Ok;
            }
            return LoginResult::Denied(code, content.msg.unwrap_or_default());
        }
    }

    LoginResult::Error
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Arc, time::Duration};

    use tokio::sync::Mutex;

    use super::*;
    use crate::{
        models::streaming::options::OptionField,
        test_support::{fixture, n},
    };

    struct MockTransport {
        responses: VecDeque<crate::Result<Option<String>>>,
        sent: Vec<String>,
        shared_sent: Arc<Mutex<Vec<String>>>,
    }

    impl MockTransport {
        fn new(responses: Vec<crate::Result<Option<String>>>) -> Self {
            Self {
                responses: VecDeque::from(responses),
                sent: Vec::new(),
                shared_sent: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn sent_handle(&self) -> Arc<Mutex<Vec<String>>> {
            self.shared_sent.clone()
        }
    }

    impl WsTransport for MockTransport {
        async fn connect(_url: &str) -> crate::Result<Self> {
            Ok(Self::new(vec![]))
        }

        async fn send(&mut self, msg: String) -> crate::Result<()> {
            self.sent.push(msg.clone());
            self.shared_sent.lock().await.push(msg);
            Ok(())
        }

        async fn next(&mut self) -> crate::Result<Option<String>> {
            if let Some(response) = self.responses.pop_front() {
                return response;
            }

            tokio::task::yield_now().await;
            Ok(None)
        }

        async fn close(&mut self) -> crate::Result<()> {
            Ok(())
        }
    }

    fn credentials() -> SessionCredentials {
        SessionCredentials {
            customer_id: "customer".to_string(),
            correl_id: "corr".to_string(),
            channel: "channel".to_string(),
            function_id: "function".to_string(),
            bearer_token: "token".to_string(),
            socket_url: "wss://example.test/stream".to_string(),
        }
    }

    async fn next_event(receiver: &mut broadcast::Receiver<StreamEvent>) -> StreamEvent {
        tokio::time::timeout(Duration::from_millis(250), receiver.recv())
            .await
            .expect("timed out waiting for stream event")
            .expect("stream event sender should remain open")
    }

    #[tokio::test]
    async fn session_login_success() {
        let transport = MockTransport::new(vec![Ok(Some(fixture("streaming_login_success.json")))]);

        let session = StreamingSession::new(transport, credentials()).await;

        assert!(session.is_ok());
    }

    #[tokio::test]
    async fn session_login_denied() {
        let transport = MockTransport::new(vec![Ok(Some(fixture("streaming_login_denied.json")))]);

        let error = match StreamingSession::new(transport, credentials()).await {
            Ok(_) => panic!("login should be denied"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            crate::Error::StreamLogin { code: 3, ref message } if message == "LOGIN_DENIED"
        ));
    }

    #[tokio::test]
    async fn session_subscribe_sends_subs_command() {
        let transport = MockTransport::new(vec![Ok(Some(fixture("streaming_login_success.json")))]);
        let sent = transport.sent_handle();
        let session = StreamingSession::new(transport, credentials())
            .await
            .unwrap();

        session
            .subscribe_equities(&["AAPL"], &[EquityField::Symbol, EquityField::BidPrice])
            .await
            .unwrap();
        let sent = sent.lock().await;
        let subscribe: serde_json::Value = serde_json::from_str(&sent[1]).unwrap();
        let item = &subscribe["requests"][0];

        assert_eq!(sent.len(), 2);
        assert_eq!(item["service"], "LEVELONE_EQUITIES");
        assert_eq!(item["command"], "SUBS");
        assert_eq!(item["parameters"]["keys"], "AAPL");
        assert_eq!(item["parameters"]["fields"], "0,1");
    }

    #[tokio::test]
    async fn session_receives_heartbeat() {
        let transport = MockTransport::new(vec![
            Ok(Some(fixture("streaming_login_success.json"))),
            Ok(Some(fixture("streaming_heartbeat.json"))),
        ]);
        let session = StreamingSession::new(transport, credentials())
            .await
            .unwrap();
        let mut events = session.subscribe();

        assert!(matches!(
            next_event(&mut events).await,
            StreamEvent::Heartbeat(1234567890)
        ));
    }

    #[tokio::test]
    async fn session_receives_equity_data() {
        let transport = MockTransport::new(vec![
            Ok(Some(fixture("streaming_login_success.json"))),
            Ok(Some(fixture("streaming_equity_data.json"))),
        ]);
        let session = StreamingSession::new(transport, credentials())
            .await
            .unwrap();
        let mut events = session.subscribe();

        let StreamEvent::Data(StreamData::LevelOneEquities(equities)) =
            next_event(&mut events).await
        else {
            panic!("expected level-one equities data event");
        };

        assert_eq!(equities.len(), 1);
        assert_eq!(equities[0].key.as_deref(), Some("AAPL"));
        assert_eq!(equities[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(equities[0].bid_price, Some(n(150.25)));
        assert_eq!(equities[0].ask_size, Some(200));
    }

    #[tokio::test]
    async fn session_receives_options_data() {
        let transport = MockTransport::new(vec![
            Ok(Some(fixture("streaming_login_success.json"))),
            Ok(Some(fixture("streaming_options_data.json"))),
        ]);
        let session = StreamingSession::new(transport, credentials())
            .await
            .unwrap();
        let mut events = session.subscribe();

        let StreamEvent::Data(StreamData::LevelOneOptions(options)) = next_event(&mut events).await
        else {
            panic!("expected level-one options data event");
        };

        assert_eq!(options.len(), 1);
        assert_eq!(options[0].symbol.as_deref(), Some("AAPL  251219C00200000"));
        assert_eq!(
            options[0].description.as_deref(),
            Some("AAPL Dec 2025 200 Call")
        );
        assert_eq!(options[0].bid_price, Some(n(5.5)));
    }

    #[tokio::test]
    async fn session_disconnect_on_transport_close() {
        let transport = MockTransport::new(vec![
            Ok(Some(fixture("streaming_login_success.json"))),
            Ok(None),
        ]);
        let session = StreamingSession::new(transport, credentials())
            .await
            .unwrap();
        let mut events = session.subscribe();

        let StreamEvent::Disconnected { error } = next_event(&mut events).await else {
            panic!("expected disconnected event");
        };

        assert_eq!(error, None);
    }

    #[test]
    fn parse_equity_data_message_maps_fields() {
        let mut messages = protocol::parse_message(&fixture("streaming_equity_data.json")).unwrap();
        let ParsedMessage::Data(data) = messages.pop().unwrap() else {
            panic!("expected data message");
        };

        let Some(StreamData::LevelOneEquities(equities)) = parse_data_message(data) else {
            panic!("expected parsed equities");
        };

        assert_eq!(equities[0].last_price, Some(n(150.4)));
        assert_eq!(equities[0].bid_size, Some(100));
    }

    #[test]
    fn parse_options_data_message_maps_fields() {
        let mut messages =
            protocol::parse_message(&fixture("streaming_options_data.json")).unwrap();
        let ParsedMessage::Data(data) = messages.pop().unwrap() else {
            panic!("expected data message");
        };

        let Some(StreamData::LevelOneOptions(options)) = parse_data_message(data) else {
            panic!("expected parsed options");
        };

        assert_eq!(options[0].ask_price, Some(n(5.7)));
        assert_eq!(options[0].high_price, Some(n(6.1)));
    }

    #[tokio::test]
    async fn session_subscribe_options_sends_subs_command() {
        let transport = MockTransport::new(vec![Ok(Some(fixture("streaming_login_success.json")))]);
        let sent = transport.sent_handle();
        let session = StreamingSession::new(transport, credentials())
            .await
            .unwrap();

        session
            .subscribe_options(
                &["AAPL  251219C00200000"],
                &[OptionField::Symbol, OptionField::BidPrice],
            )
            .await
            .unwrap();
        let sent = sent.lock().await;
        let subscribe: serde_json::Value = serde_json::from_str(&sent[1]).unwrap();
        let item = &subscribe["requests"][0];

        assert_eq!(item["service"], "LEVELONE_OPTIONS");
        assert_eq!(item["command"], "SUBS");
        assert_eq!(item["parameters"]["fields"], "0,2");
    }
}
