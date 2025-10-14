use serde::{Deserialize, Serialize};

use crate::models::board::Performance;

// ~~~~~~~~~~~~~~~~ GAME ~~~~~~~~~~~~~~~~
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
pub struct Player {
    name: String,
    id: String,
    color: String,
    rating: i32,
    //patron: Option<bool>,
    flare: Option<String>,
    online: Option<bool>,
}
