use crate::models::{AuthCallbackQuery, AuthConfig, AuthState, LichessUser, TokenInfo};
use axum::{
    Router,
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use log::info;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenResponse, TokenUrl, basic::BasicClient, reqwest::async_http_client,
};
use qrcode::{QrCode, render::unicode};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use tower_http::cors::CorsLayer;

const LICHESS_AUTH_URL: &str = "https://lichess.org/oauth";
const LICHESS_TOKEN_URL: &str = "https://lichess.org/api/token";
const LICHESS_API_BASE: &str = "https://lichess.org/api";

pub struct OAuth2Client {
    config: AuthConfig,
    client: BasicClient,
    state: Arc<Mutex<Option<AuthState>>>,
}

impl OAuth2Client {
    pub fn new(config: AuthConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let redirect_uri = format!("http://localhost:{}/callback", config.redirect_port);

        // In oauth2 v5.0, BasicClient::new only takes ClientId for PKCE flow
        let client = BasicClient::new(ClientId::new(config.client_id.clone()))
            .set_auth_uri(AuthUrl::new(LICHESS_AUTH_URL.to_string())?)
            .set_token_uri(TokenUrl::new(LICHESS_TOKEN_URL.to_string())?)
            .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

        Ok(Self {
            config,
            client,
            state: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn start_auth_flow(&self) -> Result<AuthState, Box<dyn std::error::Error>> {
        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate state token for CSRF protection
        let state = CsrfToken::new_random();

        // Build authorization URL with scopes
        let mut auth_request = self
            .client
            .authorize_url(|| state.clone())
            .set_pkce_challenge(pkce_challenge);

        for scope in &self.config.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        let (auth_url, _) = auth_request.url();

        let auth_state = AuthState {
            state: state.secret().clone(),
            code_verifier: pkce_verifier.secret().clone(),
            auth_url: auth_url.to_string(),
        };

        // Store state for callback verification
        let mut state_lock = self.state.lock().await;
        *state_lock = Some(auth_state.clone());

        Ok(auth_state)
    }

    pub async fn exchange_code(
        &self,
        code: String,
        state: String,
    ) -> Result<TokenInfo, Box<dyn std::error::Error>> {
        // Verify state
        let state_lock = self.state.lock().await;
        let stored_state = state_lock.as_ref().ok_or("No auth state found")?;

        if stored_state.state != state {
            return Err("State mismatch - possible CSRF attack".into());
        }

        let code_verifier = PkceCodeVerifier::new(stored_state.code_verifier.clone());
        drop(state_lock);

        // Exchange code for token
        let token_result = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(code_verifier)
            .request_async(async_http_client)
            .await?;

        let token_info = TokenInfo {
            access_token: token_result.access_token().secret().clone(),
            token_type: format!("{:?}", token_result.token_type()),
            expires_in: token_result.expires_in().map(|d| d.as_secs() as i64),
            scope: token_result.scopes().map(|scopes| {
                scopes
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            }),
        };

        // Clear state after successful exchange
        let mut state_lock = self.state.lock().await;
        *state_lock = None;

        Ok(token_info)
    }

    pub async fn get_user_info(
        &self,
        token: &str,
    ) -> Result<LichessUser, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/account", LICHESS_API_BASE))
            .bearer_auth(token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to get user info: {}", response.status()).into());
        }

        let user: LichessUser = response.json().await?;
        Ok(user)
    }
}

pub async fn run_auth_server(
    oauth_client: Arc<OAuth2Client>,
    shutdown_rx: oneshot::Receiver<TokenInfo>,
) -> Result<TokenInfo, Box<dyn std::error::Error>> {
    let (tx, rx) = oneshot::channel::<TokenInfo>();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let oauth_client_clone = oauth_client.clone();
    let app = Router::new()
        .route(
            "/callback",
            get(move |query: Query<AuthCallbackQuery>| {
                handle_callback(query, oauth_client_clone.clone(), tx.clone())
            }),
        )
        .route("/", get(root_handler))
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", oauth_client.config.redirect_port);
    info!("Starting OAuth callback server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Run server with graceful shutdown
    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = shutdown_rx.await;
        info!("Shutting down OAuth callback server");
    });

    // Start server in background
    tokio::spawn(async move {
        if let Err(e) = server.await {
            log::error!("Server error: {}", e);
        }
    });

    // Wait for token
    let token = rx.await?;
    Ok(token)
}

async fn handle_callback(
    Query(params): Query<AuthCallbackQuery>,
    oauth_client: Arc<OAuth2Client>,
    tx: Arc<Mutex<Option<oneshot::Sender<TokenInfo>>>>,
) -> Response {
    if let Some(error) = params.error {
        let error_msg = format!(
            "Authorization failed: {} - {}",
            error,
            params.error_description.unwrap_or_default()
        );
        info!("{}", error_msg);
        return (
            StatusCode::BAD_REQUEST,
            Html(format!(
                r#"<!DOCTYPE html>
            <html>
            <head><title>Auth Failed</title></head>
            <body>
                <h1>Authorization Failed</h1>
                <p>{}</p>
                <p>You can close this window.</p>
            </body>
            </html>"#,
                error_msg
            )),
        )
            .into_response();
    }

    let code = match params.code {
        Some(c) => c,
        None => {
            info!("No authorization code received");
            return (
                StatusCode::BAD_REQUEST,
                Html(
                    r#"<!DOCTYPE html>
                <html>
                <head><title>Auth Failed</title></head>
                <body>
                    <h1>Authorization Failed</h1>
                    <p>No authorization code received.</p>
                </body>
                </html>"#
                        .to_string(),
                ),
            )
                .into_response();
        }
    };

    let state = match params.state {
        Some(s) => s,
        None => {
            info!("No state parameter received");
            return (
                StatusCode::BAD_REQUEST,
                Html(
                    r#"<!DOCTYPE html>
                <html>
                <head><title>Auth Failed</title></head>
                <body>
                    <h1>Authorization Failed</h1>
                    <p>No state parameter received.</p>
                </body>
                </html>"#
                        .to_string(),
                ),
            )
                .into_response();
        }
    };

    info!("Received authorization code, exchanging for token...");

    match oauth_client.exchange_code(code, state).await {
        Ok(token) => {
            info!("Successfully obtained access token");

            // Send token through channel
            let mut tx_lock = tx.lock().await;
            if let Some(sender) = tx_lock.take() {
                let _ = sender.send(token);
            }

            (
                StatusCode::OK,
                Html(
                    r#"<!DOCTYPE html>
                <html>
                <head>
                    <title>Auth Success</title>
                    <style>
                        body { font-family: Arial; padding: 40px; text-align: center; }
                        .success { color: green; }
                    </style>
                </head>
                <body>
                    <h1 class="success">✓ Authorization Successful!</h1>
                    <p>You have successfully authenticated with Lichess.</p>
                    <p>You can now close this window and return to your application.</p>
                </body>
                </html>"#
                        .to_string(),
                ),
            )
                .into_response()
        }
        Err(e) => {
            info!("Failed to exchange code: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    r#"<!DOCTYPE html>
                <html>
                <head><title>Auth Failed</title></head>
                <body>
                    <h1>Authorization Failed</h1>
                    <p>Failed to exchange code: {}</p>
                </body>
                </html>"#,
                    e
                )),
            )
                .into_response()
        }
    }
}

async fn root_handler() -> Html<String> {
    Html(
        r#"<!DOCTYPE html>
        <html>
        <head><title>Lichess OAuth</title></head>
        <body>
            <h1>Lichess OAuth Server</h1>
            <p>Waiting for OAuth callback...</p>
        </body>
        </html>"#
            .to_string(),
    )
}

pub fn generate_qr_code(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let code = QrCode::new(url)?;
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    Ok(image)
}

pub async fn authenticate() -> Result<(TokenInfo, LichessUser), Box<dyn std::error::Error>> {
    let config = AuthConfig::default();
    let oauth_client = Arc::new(OAuth2Client::new(config)?);

    // Start auth flow
    let auth_state = oauth_client.start_auth_flow().await?;

    info!("Starting OAuth2 authentication flow...");
    info!("Please visit the following URL to authenticate:");
    info!("{}", auth_state.auth_url);

    // Generate QR code for mobile authentication
    match generate_qr_code(&auth_state.auth_url) {
        Ok(qr) => {
            info!("Or scan this QR code with your mobile device:");
            info!("\n{}", qr);
        }
        Err(e) => {
            info!("Could not generate QR code: {}", e);
        }
    }

    // Start callback server and wait for auth
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<TokenInfo>();
    let token = run_auth_server(oauth_client.clone(), shutdown_rx).await?;

    // Notify shutdown
    let _ = shutdown_tx.send(token.clone());

    // Get user info
    let user = oauth_client.get_user_info(&token.access_token).await?;
    info!("Successfully authenticated as: {}", user.username);

    Ok((token, user))
}
