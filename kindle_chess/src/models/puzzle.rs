use serde::{Deserialize, Serialize};

use crate::models::game::Game;

// PUZZLES
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
pub struct DailyPuzzle {
    pub game: Game,
    pub puzzle: Puzzle,
}
