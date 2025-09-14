use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use tower_http::cors::CorsLayer;

use axum::{Router, routing::get};

use crate::api::models::OAuthClient;

impl OAuthClient {
    pub fn new(client_id: String, client_secret: String) -> Self {
        let client = BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_auth_uri(AuthUrl::new("https:lichess.org/oauth".to_string()).unwrap())
            .set_token_uri(TokenUrl::new("https://lichess.org/api/token".to_string()).unwrap())
            .set_redirect_uri(
                RedirectUrl::new("https://localhost:8080/callback".to_string()).unwrap(),
            );

        Self {
            client,
            pkce_verifier: Arc::new(Mutex::new(None)),
            csrf_token: Arc::new(Mutex::new(None)),
            access_token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn start_oauth_flow() -> Result<String, Box<dyn std::error::Error>> {
        // Start local server
        self.start_server();
        // Wait for flow to complete
        self.wait_for_token();
    }

    async fn start_server(&self) {
        let oauth_client = self.clone();

        let app = Router::new()
            .route(
                "login",
                get(move || {
                    let client = oauth_client.clone();
                    async move { client.handle_login().await }
                }),
            )
            .route(
                "/callback",
                get({
                    let client = self.clone();
                    move |query| {
                        let client = client.clone();
                        async move { client.jandle_callback(query).await }
                    }
                }),
            )
            .layer(CorsLayer::permissive());
    }
}
