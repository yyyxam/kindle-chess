use crate::api::models::{
    AuthCallbackQuery, AuthConfig, AuthState, LichessUser, OAuth2Client, TokenInfo,
};
use axum::{
    Router,
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use log::info;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, EmptyExtraTokenFields, EndpointNotSet,
    EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RevocationErrorResponseType,
    Scope, StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse, TokenUrl,
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
};
use qrcode::{QrCode, render::unicode};
use reqwest::ClientBuilder;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use tower_http::cors::CorsLayer;

use std::fs::{File, remove_file, write};
use std::io::prelude::Read;

impl OAuth2Client {
    pub fn new(config: AuthConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let host_ip = local_ip_address::local_ip()?.to_string();
        let redirect_uri = format!("http://{}:{}/callback", host_ip, config.redirect_port);

        Ok(Self {
            client_id: ClientId::new(config.client_id.clone()),
            redirect_url: RedirectUrl::new(redirect_uri)?,
            auth_url: AuthUrl::new("https://lichess.org/oauth".to_string())?,
            token_url: TokenUrl::new(concat!(env!("LICHESS_API_BASE"), "/token").to_string())?,
            config,
            state: Arc::new(Mutex::new(None::<AuthState>)),
        })
    }

    fn create_client(
        &self,
    ) -> oauth2::Client<
        StandardErrorResponse<BasicErrorResponseType>,
        StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
        StandardRevocableToken,
        StandardErrorResponse<RevocationErrorResponseType>,
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointSet,
    > {
        BasicClient::new(self.client_id.clone())
            .set_auth_uri(self.auth_url.clone())
            .set_token_uri(self.token_url.clone())
            .set_redirect_uri(self.redirect_url.clone())
    }

    fn create_http_client(&self) -> reqwest::Client {
        ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build.")
    }

    pub async fn start_auth_flow(&self) -> Result<AuthState, Box<dyn std::error::Error>> {
        // Create a fresh client for this request
        let client = self.create_client();

        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate state token for CSRF protection
        let state = CsrfToken::new_random();

        // Build authorization URL with scopes
        let mut auth_request = client
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
    ) -> Result<TokenInfo, Box<dyn std::error::Error + Send + Sync>> {
        // Verify state
        let state_lock = self.state.lock().await;
        let stored_state = state_lock.as_ref().ok_or("No auth state found")?;

        if stored_state.state != state {
            return Err("State mismatch - possible CSRF attack".into());
        }

        let code_verifier = PkceCodeVerifier::new(stored_state.code_verifier.clone());
        drop(state_lock);

        // Create a fresh client for token exchange
        let client = self.create_client();
        let http_client = self.create_http_client();

        // Exchange code for token
        let token_result = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(code_verifier)
            .request_async(&http_client)
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
}

pub async fn run_auth_server(
    oauth_client: Arc<OAuth2Client>,
    shutdown_rx: oneshot::Receiver<TokenInfo>,
) -> Result<TokenInfo, Box<dyn std::error::Error>> {
    let (tx, rx) = oneshot::channel::<TokenInfo>();

    let app = Router::new()
        .route(
            "/callback",
            get({
                let tx = Arc::new(Mutex::new(Some(tx)));
                let oauth_client_clone = oauth_client.clone();
                async move |query: Query<AuthCallbackQuery>| {
                    handle_callback(query, oauth_client_clone.clone(), tx.clone()).await
                }
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

pub async fn get_user_info(token: &str) -> Result<LichessUser, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/account", env!("LICHESS_API_BASE")))
        .bearer_auth(token)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Failed to get user info: {}", response.status()).into());
    }

    let user: LichessUser = response.json().await?;
    Ok(user)
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
    let user = get_user_info(&token.access_token).await?;
    info!("Successfully re-authenticated as: {}", user.username);

    // Write TokenInfo to token.json-File
    match write(env!("AUTH_TOKEN"), serde_json::to_string_pretty(&token)?) {
        Ok(()) => {
            info!("Auth-Token written to  {}", env!("AUTH_TOKEN"))
        }
        Err(e) => {
            info!("Error writing AuthToken: {}", e)
        }
    }

    Ok((token, user))
}

pub async fn load_token() -> Result<(TokenInfo, LichessUser), Box<dyn std::error::Error>> {
    let mut file = File::open(std::path::Path::new(env!("AUTH_TOKEN")))?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    let token_info = serde_json::from_slice::<TokenInfo>(&buf[..])?;
    let user = get_user_info(&token_info.access_token).await?;
    info!(
        "Successfully authenticated from token as: {}",
        user.username
    );
    return Ok((token_info, user));
}

pub async fn get_authenticated() -> Result<String, Box<dyn std::error::Error>> {
    match load_token().await {
        Ok((token_info, _)) => {
            info!("Authenticated via Token - Nice!");
            return Ok(token_info.access_token);
        }
        Err(_) => {
            //authenticate
            info!("No token found..");
            let (token_info, _) = authenticate().await?;
            info!("Authenticated via direct authentification - Nice!");
            return Ok(token_info.access_token);
        }
    }
}

pub fn logout() -> std::io::Result<()> {
    remove_file(env!("AUTH_TOKEN"))?;
    Ok(())
}
