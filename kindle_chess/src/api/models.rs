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

// GAME
#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    id: String,
    perf: Performance,
    rated: bool,
    players: Vec<Player>,
    pgn: String,
    clock: String,
}

// BOARD

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum StreamEvent {
    GameStart(GameStartEvent),
    GameFinish(GameFinishEvent),
    Challenge(ChallengeEvent),
    ChallengeDeclined(ChallengeDeclinedEvent),
}

// EVENT-STREAM-TYPES
#[derive(Serialize, Deserialize, Debug)]
pub struct GameStartEvent {
    #[serde(rename = "fullId")]
    pub full_id: String,
    #[serde(rename = "gameId")]
    pub game_id: String,
    pub fen: String,
    pub color: String,
    #[serde(rename = "lastMove")]
    pub last_move: String,
    pub source: String,
    pub status: GameStatus,
    pub variant: GameVariant,
    pub speed: String,
    pub perf: String,
    pub rated: bool,
    #[serde(rename = "hasMoved")]
    pub has_moved: bool,
    pub opponent: Opponent,
    #[serde(rename = "isMyTurn")]
    pub is_my_turn: bool,
    #[serde(rename = "secondsLeft")]
    pub seconds_left: Option<u64>,
    pub compat: Compat,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameFinishEvent {
    #[serde(rename = "fullId")]
    pub full_id: String,
    #[serde(rename = "gameId")]
    pub game_id: String,
    pub fen: String,
    pub color: String,
    #[serde(rename = "lastMove")]
    pub last_move: String,
    pub source: String,
    pub status: GameStatus,
    pub variant: GameVariant,
    pub speed: String,
    pub perf: String,
    pub rated: bool,
    #[serde(rename = "hasMoved")]
    pub has_moved: bool,
    pub opponent: Opponent,
    #[serde(rename = "isMyTurn")]
    pub is_my_turn: bool,
    #[serde(rename = "secondsLeft")]
    pub seconds_left: Option<u64>,
    pub winner: Option<String>,
    #[serde(rename = "ratingDiff")]
    pub rating_diff: Option<i16>,
    pub compat: Compat,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChallengeEvent {
    pub id: String,
    pub url: String,
    pub status: String,
    pub challenger: Player,
    #[serde(rename = "destUser")]
    pub dest_user: Player,
    pub variant: GameVariant,
    pub rated: bool,
    pub speed: String,
    #[serde(rename = "timeControl")]
    pub time_control: TimeControl,
    pub color: String,
    #[serde(rename = "finalColor")]
    pub final_color: String,
    pub perf: PerfCallenge,
    pub compat: Compat,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChallengeDeclinedEvent {
    pub id: String,
    pub url: String,
    pub status: String,
    pub challenger: Player,
    #[serde(rename = "destUser")]
    pub dest_user: Player,
    pub variant: GameVariant,
    pub rated: bool,
    pub speed: String,
    #[serde(rename = "timeControl")]
    pub time_control: TimeControl,
    pub color: String,
    #[serde(rename = "finalColor")]
    pub final_color: String,
    pub perf: PerfCallenge,
    pub compat: Compat,
    #[serde(rename = "declineReason")]
    decline_reason: String,
    #[serde(rename = "declineReasonKey")]
    decline_reason_key: String,
}

// MISC EVENT-STREAM-TYPES
#[derive(Serialize, Deserialize, Debug)]
pub struct GameStatus {
    id: u16,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Opponent {
    id: String,
    username: String,
    rating: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Compat {
    bot: bool,
    board: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeControl {
    #[serde(rename = "type")]
    tc_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PerfCallenge {
    icon: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum GameStateStreamEvent {
    GameFull(GameFullEvent),
    GameState(GameStateEvent),
    ChatLine(ChatLineEvent),
    OpponentGone(OpponentGoneEvent),
}

// GAME-STATE-STREAM-EVENT-TYPES
#[derive(Serialize, Deserialize, Debug)]
pub struct GameFullEvent {
    pub id: String,
    pub variant: GameVariant,
    pub speed: String,
    pub perf: PerfMode,
    pub rated: bool,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub white: PlayedBy,
    pub black: PlayedBy,
    #[serde(rename = "initialFen")]
    pub initial_fen: String,
    pub clock: Clock,
    // #[serde(rename = "type")]
    // pub event_type: String,
    pub state: GameStateEvent,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameStateEvent {
    // #[serde(rename = "type")]
    // pub event_type: String,
    pub moves: String,
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    //pub wdraw: bool,
    //pub bdraw: bool,
    // pub wtakeback: bool,
    // pub btakeback: bool,
    // pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatLineEvent {
    // #[serde(rename = "type")]
    // pub event_type: String,
    pub username: String,
    pub text: String,
    pub room: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpponentGoneEvent {
    // #[serde(rename = "type")]
    // pub event_type: String,
    pub gone: bool,
    #[serde(rename = "claimWinInSeconds")]
    pub claim_win_in_seconds: u64,
}

// MISC GAME-STATE-STREAM-EVENT-TYPES
#[derive(Serialize, Deserialize, Debug)]
pub struct GameVariant {
    key: String,
    name: String,
    short: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PerfMode {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Clock {
    initial: u64,
    increment: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayedBy {
    id: String,
    name: String,
    title: Option<String>,
    rating: u64,
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
pub struct Player {
    name: String,
    id: String,
    color: String,
    rating: i32,
    //patron: Option<bool>,
    flare: Option<String>,
    online: Option<bool>,
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

#[derive(Debug, Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    STREAM,
}
