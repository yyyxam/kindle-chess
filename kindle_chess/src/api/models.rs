use std::sync::Arc;
use tokio::sync::Mutex;

use ::uuid::Uuid;
use oauth2::{AccessToken, AuthUrl, ClientId, RedirectUrl, TokenUrl};
use serde::{Deserialize, Serialize};

// PUZZLES
#[derive(Serialize, Deserialize, Debug)]
pub struct DailyPuzzle {
    pub game: Game,
    pub puzzle: Puzzle,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    id: String,
    perf: Performance,
    rated: bool,
    players: Vec<Player>,
    pgn: String,
    clock: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Puzzle {
    id: String,
    rating: i32,
    plays: i32,
    solution: Vec<String>,
    themes: Vec<String>,
    #[serde(rename = "initialPly")]
    initial_play: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    name: String,
    id: String,
    color: String,
    rating: i32,
    //patron: Option<bool>,
    //flare: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Performance {
    key: String,
    name: String,
}

// OAUTH2
pub struct OAuth2Client {
    pub config: AuthConfig,
    pub client_id: ClientId,
    pub redirect_url: RedirectUrl,
    pub auth_url: AuthUrl,
    pub token_url: TokenUrl,
    pub state: Arc<Mutex<Option<AuthState>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LichessUser {
    pub id: String,
    pub username: String,
    pub perfs: Option<serde_json::Value>,
    pub created_at: Option<i64>,
    pub disabled: Option<bool>,
    pub tos_violation: Option<bool>,
    pub profile: Option<UserProfile>,
    pub seen_at: Option<i64>,
    pub patron: Option<bool>,
    pub verified: Option<bool>,
    pub play_time: Option<serde_json::Value>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub country: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub links: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub scope: Option<String>,
}

impl TokenInfo {
    pub fn to_oauth2_token(&self) -> AccessToken {
        AccessToken::new(self.access_token.clone())
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub client_id: String,
    pub redirect_port: u16,
    pub scopes: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            client_id: format!("lichess-rust-client-{}", Uuid::new_v4()),
            redirect_port: 8080,
            scopes: vec![
                "challenge:read".to_string(),
                "challenge:write".to_string(),
                "bot:play".to_string(),
                "board:play".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthState {
    pub state: String,
    pub code_verifier: String,
    pub auth_url: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}
