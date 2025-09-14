use oauth2::basic::BasicClient;
use oauth2::{CsrfToken, PkceCodeVerifier};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

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
#[derive(Deserialize, Debug)]
pub struct CallbackParams {
    code: String,
    state: String,
}

pub struct OAuthClient {
    client: BasicClient,
    pkce_verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
    csrf_token: Arc<Mutex<Option<CsrfToken>>>,
    access_token: Arc<Mutex<Option<String>>>,
}
