use std::collections::HashMap;

use reqwest::Method;
use tracing::instrument;

use crate::client::ApiBase;
use crate::models::market_data::{
    CandleList, ExpirationChain, Hours, InstrumentResponse, InstrumentsResponse, OptionChain,
    ScreenerResponse,
};
use crate::query::{
    comma_separated_required, comma_separated_symbols, push_optional, required_text,
};
use crate::{
    Client, MoverOptions, OptionChainOptions, PriceHistoryOptions, QuoteOptions, Quotes, Result,
};

impl Client {
    /// Fetches quotes for a list of symbols.
    #[instrument(skip_all)]
    pub async fn get_quotes<S>(&self, symbols: impl IntoIterator<Item = S>) -> Result<Quotes>
    where
        S: AsRef<str>,
    {
        self.get_quotes_with_options(symbols, QuoteOptions::default())
            .await
    }

    /// Fetches quotes with optional OpenAPI query parameters.
    #[instrument(skip_all)]
    pub async fn get_quotes_with_options<S>(
        &self,
        symbols: impl IntoIterator<Item = S>,
        options: QuoteOptions,
    ) -> Result<Quotes>
    where
        S: AsRef<str>,
    {
        let symbols = comma_separated_symbols(symbols)?;
        let url = self.endpoint_url(ApiBase::MarketData, &["quotes"])?;
        let mut query = vec![("symbols", symbols)];
        push_optional(&mut query, "fields", options.fields);
        if options.indicative {
            query.push(("indicative", options.indicative.to_string()));
        }
        self.send_json(Method::GET, url, &query, None).await
    }

    /// Fetches the single-symbol quote endpoint from `GET /{symbol_id}/quotes`.
    #[instrument(skip_all)]
    pub async fn get_quote(
        &self,
        symbol_id: impl AsRef<str>,
        fields: Option<&str>,
    ) -> Result<Quotes> {
        let symbol_id = required_text("symbol_id", symbol_id.as_ref())?;
        let url = self.endpoint_url(ApiBase::MarketData, &[&symbol_id, "quotes"])?;
        let mut query = Vec::new();
        push_optional(&mut query, "fields", fields);
        self.send_json(Method::GET, url, &query, None).await
    }

    /// Fetches an option chain from `GET /chains`.
    #[instrument(skip_all)]
    pub async fn get_option_chain(&self, options: OptionChainOptions) -> Result<OptionChain> {
        required_text("symbol", &options.symbol)?;
        let url = self.endpoint_url(ApiBase::MarketData, &["chains"])?;
        self.send_json(Method::GET, url, &options.into_query(), None)
            .await
    }

    /// Fetches option expirations from `GET /expirationchain`.
    #[instrument(skip_all)]
    pub async fn get_expiration_chain(&self, symbol: impl AsRef<str>) -> Result<ExpirationChain> {
        let symbol = required_text("symbol", symbol.as_ref())?;
        let url = self.endpoint_url(ApiBase::MarketData, &["expirationchain"])?;
        self.send_json(Method::GET, url, &[("symbol", symbol.to_owned())], None)
            .await
    }

    /// Searches instruments from `GET /instruments`.
    #[instrument(skip_all)]
    pub async fn get_instruments(
        &self,
        symbol: impl AsRef<str>,
        projection: impl AsRef<str>,
    ) -> Result<InstrumentsResponse> {
        let symbol = required_text("symbol", symbol.as_ref())?;
        let projection = required_text("projection", projection.as_ref())?;
        let url = self.endpoint_url(ApiBase::MarketData, &["instruments"])?;
        self.send_json(
            Method::GET,
            url,
            &[
                ("symbol", symbol.to_owned()),
                ("projection", projection.to_owned()),
            ],
            None,
        )
        .await
    }

    /// Fetches a single instrument from `GET /instruments/{cusip_id}`.
    #[instrument(skip_all)]
    pub async fn get_instrument_by_cusip(
        &self,
        cusip_id: impl AsRef<str>,
    ) -> Result<InstrumentResponse> {
        let cusip_id = required_text("cusip_id", cusip_id.as_ref())?;
        let url = self.endpoint_url(ApiBase::MarketData, &["instruments", &cusip_id])?;
        self.send_json(Method::GET, url, &[], None).await
    }

    /// Fetches market hours from `GET /markets`.
    #[instrument(skip_all)]
    pub async fn get_market_hours<S>(
        &self,
        markets: impl IntoIterator<Item = S>,
        date: Option<&str>,
    ) -> Result<HashMap<String, HashMap<String, Hours>>>
    where
        S: AsRef<str>,
    {
        let markets = comma_separated_required("markets", markets)?;
        let url = self.endpoint_url(ApiBase::MarketData, &["markets"])?;
        let mut query = vec![("markets", markets)];
        push_optional(&mut query, "date", date);
        self.send_json(Method::GET, url, &query, None).await
    }

    /// Fetches market hours from `GET /markets/{market_id}`.
    #[instrument(skip_all)]
    pub async fn get_market_hour(
        &self,
        market_id: impl AsRef<str>,
        date: Option<&str>,
    ) -> Result<HashMap<String, HashMap<String, Hours>>> {
        let market_id = required_text("market_id", market_id.as_ref())?;
        let url = self.endpoint_url(ApiBase::MarketData, &["markets", &market_id])?;
        let mut query = Vec::new();
        push_optional(&mut query, "date", date);
        self.send_json(Method::GET, url, &query, None).await
    }

    /// Fetches market movers from `GET /movers/{symbol_id}`.
    #[instrument(skip_all)]
    pub async fn get_movers(
        &self,
        symbol_id: impl AsRef<str>,
        options: MoverOptions,
    ) -> Result<ScreenerResponse> {
        let symbol_id = required_text("symbol_id", symbol_id.as_ref())?;
        let url = self.endpoint_url(ApiBase::MarketData, &["movers", &symbol_id])?;
        self.send_json(Method::GET, url, &options.into_query(), None)
            .await
    }

    /// Fetches price history from `GET /pricehistory`.
    #[instrument(skip_all)]
    pub async fn get_price_history(
        &self,
        symbol: impl AsRef<str>,
        options: PriceHistoryOptions,
    ) -> Result<CandleList> {
        let symbol = required_text("symbol", symbol.as_ref())?;
        let url = self.endpoint_url(ApiBase::MarketData, &["pricehistory"])?;
        let mut query = vec![("symbol", symbol.to_owned())];
        query.extend(options.into_query());
        self.send_json(Method::GET, url, &query, None).await
    }
}

#[cfg(test)]
mod tests {
    use mockito::Matcher;

    use crate::test_support::fixture;
    use crate::*;

    #[tokio::test]
    async fn get_quotes_sends_expected_no_auth_request() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/quotes")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("symbols".into(), "AAPL,MSFT".into()),
                Matcher::UrlEncoded("fields".into(), "quote,reference".into()),
                Matcher::UrlEncoded("indicative".into(), "true".into()),
            ]))
            .match_header("authorization", Matcher::Missing)
            .with_status(200)
            .with_body(fixture("quote_equity.json"))
            .create_async()
            .await;

        let url = server.url();
        let client = Client::new(Config::new().base_url(&url).unwrap());
        let quotes = client
            .get_quotes_with_options(
                ["AAPL", "MSFT"],
                QuoteOptions::new()
                    .fields("quote,reference")
                    .indicative(true),
            )
            .await
            .unwrap();

        mock.assert_async().await;
        assert!(quotes.contains_key("AAPL"));
    }

    #[tokio::test]
    async fn get_quotes_returns_http_status_errors() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", Matcher::Any)
            .with_status(400)
            .with_body(r#"{"message":"bad symbols"}"#)
            .create_async()
            .await;

        let url = server.url();
        let client = Client::new(Config::new().base_url(&url).unwrap());
        let error = client.get_quotes(["AAPL"]).await.unwrap_err();

        assert!(matches!(
            error,
            Error::HttpStatus { status: 400, body } if body.contains("bad symbols")
        ));
    }

    #[tokio::test]
    async fn get_option_chain_sends_documented_query_parameters() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/chains")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
                Matcher::UrlEncoded("contractType".into(), "CALL".into()),
                Matcher::UrlEncoded("strikeCount".into(), "5".into()),
                Matcher::UrlEncoded("includeUnderlyingQuote".into(), "true".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"status":"SUCCESS"}"#)
            .create_async()
            .await;

        let url = server.url();
        let client = Client::new(Config::new().base_url(&url).unwrap());
        client
            .get_option_chain(
                OptionChainOptions::new("AAPL")
                    .parameter("contractType", "CALL")
                    .integer_parameter("strikeCount", 5)
                    .include_underlying_quote(true),
            )
            .await
            .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn numeric_option_helpers_preserve_explicit_zero_values() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/chains")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
                Matcher::UrlEncoded("strikeCount".into(), "0".into()),
                Matcher::UrlEncoded("volatility".into(), "0".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"status":"SUCCESS"}"#)
            .create_async()
            .await;

        let url = server.url();
        let client = Client::new(Config::new().base_url(&url).unwrap());
        client
            .get_option_chain(
                OptionChainOptions::new("AAPL")
                    .integer_parameter("strikeCount", 0)
                    .number_parameter("volatility", 0.0),
            )
            .await
            .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn endpoint_request_shapes_are_covered() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();
        let client = Client::new(Config::new().base_url(&url).unwrap());

        // get_quote with path-encoded symbol
        let m = server
            .mock("GET", "/AAPL%2FPR/quotes")
            .match_query(Matcher::UrlEncoded("fields".into(), "quote".into()))
            .with_status(200)
            .with_body(r#"{"AAPL":{"assetMainType":"EQUITY"}}"#)
            .create_async()
            .await;
        client.get_quote("AAPL/PR", Some("quote")).await.unwrap();
        m.assert_async().await;

        // get_expiration_chain
        let m = server
            .mock("GET", "/expirationchain")
            .match_query(Matcher::UrlEncoded("symbol".into(), "AAPL".into()))
            .with_status(200)
            .with_body(r#"{"expirationList":[]}"#)
            .create_async()
            .await;
        client.get_expiration_chain("AAPL").await.unwrap();
        m.assert_async().await;

        // get_instruments
        let m = server
            .mock("GET", "/instruments")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
                Matcher::UrlEncoded("projection".into(), "symbol-search".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"instruments":[]}"#)
            .create_async()
            .await;
        client
            .get_instruments("AAPL", "symbol-search")
            .await
            .unwrap();
        m.assert_async().await;

        // get_instrument_by_cusip with path encoding
        let m = server
            .mock("GET", "/instruments/CUSIP%2F1")
            .with_status(200)
            .with_body(r#"{"cusip":"037833100"}"#)
            .create_async()
            .await;
        client.get_instrument_by_cusip("CUSIP/1").await.unwrap();
        m.assert_async().await;

        // get_market_hours
        let m = server
            .mock("GET", "/markets")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("markets".into(), "equity,option".into()),
                Matcher::UrlEncoded("date".into(), "2024-01-02".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"equity":{}}"#)
            .create_async()
            .await;
        client
            .get_market_hours(["equity", "option"], Some("2024-01-02"))
            .await
            .unwrap();
        m.assert_async().await;

        // get_market_hour
        let m = server
            .mock("GET", "/markets/equity")
            .match_query(Matcher::UrlEncoded("date".into(), "2024-01-02".into()))
            .with_status(200)
            .with_body(r#"{"equity":{"EQ":{}}}"#)
            .create_async()
            .await;
        client
            .get_market_hour("equity", Some("2024-01-02"))
            .await
            .unwrap();
        m.assert_async().await;

        // get_movers
        let m = server
            .mock("GET", "/movers/$DJI")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("sort".into(), "VOLUME".into()),
                Matcher::UrlEncoded("frequency".into(), "5".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"screeners":[]}"#)
            .create_async()
            .await;
        client
            .get_movers("$DJI", MoverOptions::new().sort("VOLUME").frequency(5))
            .await
            .unwrap();
        m.assert_async().await;

        // get_price_history
        let m = server
            .mock("GET", "/pricehistory")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
                Matcher::UrlEncoded("periodType".into(), "day".into()),
                Matcher::UrlEncoded("period".into(), "0".into()),
                Matcher::UrlEncoded("needExtendedHoursData".into(), "false".into()),
            ]))
            .with_status(200)
            .with_body(r#"{"candles":[]}"#)
            .create_async()
            .await;
        client
            .get_price_history(
                "AAPL",
                PriceHistoryOptions::new()
                    .parameter("periodType", "day")
                    .integer_parameter("period", 0)
                    .bool_parameter("needExtendedHoursData", false),
            )
            .await
            .unwrap();
        m.assert_async().await;
    }

    #[tokio::test]
    async fn required_list_rejects_empty_values() {
        let client = Client::new(Config::new());

        let result = client.get_market_hours([" "], None).await;
        assert!(matches!(
            result,
            Err(Error::MissingRequiredParameter("markets"))
        ));
    }
}
