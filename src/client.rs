use reqwest::Method;
use serde::{Serialize, de::DeserializeOwned};

use crate::{Config, Error, OrderResponse, Result};

/// Schwab API client.
#[derive(Clone)]
pub struct Client {
    pub(crate) config: Config,
    http: reqwest::Client,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Client")
            .field("config", &self.config)
            .field("http", &"<reqwest::Client>")
            .finish()
    }
}

impl Client {
    /// Creates a client from configuration.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    pub(crate) fn endpoint_url(
        &self,
        api_base: ApiBase,
        path_segments: &[&str],
    ) -> Result<reqwest::Url> {
        let raw_base_url = match api_base {
            ApiBase::MarketData => &self.config.market_data_base_url,
            ApiBase::Trader => &self.config.trader_base_url,
        };
        let base_url = format!("{raw_base_url}/");
        let mut url = reqwest::Url::parse(&base_url).map_err(|error| Error::InvalidBaseUrl {
            base_url: raw_base_url.clone(),
            message: error.to_string(),
        })?;

        {
            let mut segments = url
                .path_segments_mut()
                .map_err(|()| Error::InvalidBaseUrl {
                    base_url: raw_base_url.clone(),
                    message: "base URL cannot be a base for path segments".to_string(),
                })?;
            for segment in path_segments {
                segments.push(segment);
            }
        }

        Ok(url)
    }

    pub(crate) async fn send_json<T>(
        &self,
        method: Method,
        url: reqwest::Url,
        query: &[(&str, String)],
        body: Option<serde_json::Value>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self.send(method, url, query, body).await?;
        let text = response.text().await.map_err(Error::Request)?;
        serde_json::from_str::<T>(&text).map_err(|source| Error::Decode { source, body: text })
    }

    pub(crate) async fn send_empty(&self, method: Method, url: reqwest::Url) -> Result<()> {
        self.send(method, url, &[], None).await.map(|_| ())
    }

    pub(crate) async fn send_empty_with_location<B>(
        &self,
        method: Method,
        url: reqwest::Url,
        body: &B,
    ) -> Result<OrderResponse>
    where
        B: Serialize + ?Sized,
    {
        let response = self
            .send(
                method,
                url,
                &[],
                Some(serde_json::to_value(body).map_err(Error::Encode)?),
            )
            .await?;
        Ok(OrderResponse::from_location_header(response.headers()))
    }

    async fn send(
        &self,
        method: Method,
        url: reqwest::Url,
        query: &[(&str, String)],
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response> {
        let mut request = self
            .http
            .request(method, url)
            .header(reqwest::header::ACCEPT, "application/json");

        if let Some(token) = &self.config.bearer_token {
            request = request.bearer_auth(token);
        }
        if !query.is_empty() {
            request = request.query(&query);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await.map_err(Error::Request)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.map_err(Error::Request)?;
            return Err(Error::HttpStatus {
                status: status.as_u16(),
                body,
            });
        }
        Ok(response)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApiBase {
    MarketData,
    Trader,
}

#[cfg(test)]
mod tests {
    use std::error::Error as StdError;
    use std::net::TcpListener;

    use crate::*;

    #[test]
    fn debug_output_redacts_bearer_token() {
        let client = Client::new(Config::new().bearer_token("SECRET_TOKEN"));
        let debug_output = format!("{client:?}");

        assert!(debug_output.contains("<redacted>"));
        assert!(!debug_output.contains("SECRET_TOKEN"));
    }

    #[tokio::test]
    async fn request_and_decode_errors_expose_sources_and_messages() {
        // Decode error: server returns 200 with non-JSON body
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body("not-json")
            .create_async()
            .await;

        let url = server.url();
        let client = Client::new(Config::new().base_url(&url).unwrap());
        let decode_error = client.get_quotes(["AAPL"]).await.unwrap_err();

        assert!(
            decode_error
                .to_string()
                .starts_with("failed to decode Schwab response:")
        );
        assert!(StdError::source(&decode_error).is_some());

        // Request error: connection refused on closed port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let closed_base_url = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        let client = Client::new(Config::new().base_url(closed_base_url).unwrap());
        let request_error = client.get_quotes(["AAPL"]).await.unwrap_err();

        assert!(
            request_error
                .to_string()
                .starts_with("HTTP request failed:")
        );
        assert!(StdError::source(&request_error).is_some());
    }

    #[tokio::test]
    async fn decode_error_preserves_raw_body() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body("not valid json")
            .create_async()
            .await;

        let url = server.url();
        let client = Client::new(Config::new().base_url(&url).unwrap());
        let error = client.get_quotes(["AAPL"]).await.unwrap_err();

        match &error {
            Error::Decode { body, .. } => assert_eq!(body, "not valid json"),
            other => panic!("expected Decode, got {other:?}"),
        }

        // Debug output redacts the body
        let debug = format!("{error:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("not valid json"));
    }
}
