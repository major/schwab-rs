# Schwab OAuth login

Schwab requires a user to approve access in a browser, then redirects back to a localhost callback URL. The Rust auth module implements that flow without making the main API client own refresh state.

## Flow

1. Register a Schwab callback URL like `https://127.0.0.1:8182/callback`.
2. Build `auth::AuthConfig` with the app key, app secret, and callback URL.
3. Start `auth::login(...)` or `auth::start_login(...)` with a `TokenStore`.
4. Open or print `AuthContext::authorization_url`.
5. Complete the browser approval. Schwab redirects to the generated HTTPS localhost server.
6. The auth module validates `state`, exchanges the authorization code, and saves `{ "creation_timestamp": ..., "token": ... }`.
7. Use `Provider::client().await?` when you need a regular `Client` snapshot with the current bearer token.

## Example

```bash
SCHWAB_CLIENT_ID='your-app-key' \
SCHWAB_CLIENT_SECRET='your-app-secret' \
SCHWAB_CALLBACK_URL='https://127.0.0.1:8182/callback' \
SCHWAB_TOKEN_PATH='schwab-token.json' \
cargo run --example auth
```

The callback URL must use `https`, host `127.0.0.1`, and include an explicit port. The server binds only to loopback, generates an in-memory self-signed certificate for the callback listener, validates the OAuth state exactly, and writes token files atomically with owner-only permissions on Unix.
