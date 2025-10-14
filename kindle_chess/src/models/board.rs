use serde::{Deserialize, Serialize};

use crate::models::{
    game::Player,
    oauth::{LichessUser, TokenInfo},
};

// BOARD
#[derive(Debug, Deserialize, Serialize)]
pub struct Board {
    pub token: TokenInfo,
    pub user: LichessUser, // == player0
    pub bitboard: Vec<u64>,
    pub game_id: String,
    pub white: Option<PlayedBy>,
    pub black: Option<PlayedBy>,
    pub player0_white: bool, // if true then player0 had first turn
    pub player0_turn: bool,  // if true then it's currently player0's turn
}

// ~~~~~~~~~~~~~~ STREAMS ~~~~~~~~~~~~~~~

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
    // pub id: Option<String>,
    pub variant: GameVariant,
    pub speed: Speed,
    pub perf: PerfMode,
    pub rated: bool,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub white: PlayedBy,
    pub black: PlayedBy,
    #[serde(rename = "initialFen")]
    pub initial_fen: String,
    pub clock: Option<Clock>,
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
    pub wdraw: Option<bool>,
    pub bdraw: Option<bool>,
    pub wtakeback: Option<bool>,
    pub btakeback: Option<bool>,
    pub status: String,
    pub winner: Option<String>,
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum PlayedBy {
    User(PlayedByPlayer),
    Ai(PlayedByAi),
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct PlayedByPlayer {
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub rating: u64,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct PlayedByAi {
    #[serde(rename = "aiLevel")]
    pub ai_level: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Performance {
    key: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Speed {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}
