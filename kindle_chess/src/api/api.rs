use crate::api::models::DailyPuzzle;
use log::info;
use reqwest::Url;

// PUZZLE-ENDPOINT
// unpriviliged
pub async fn get_daily_puzzle() -> Result<DailyPuzzle, Box<dyn std::error::Error>> {
    let url = format!("{}/puzzle/daily", env!("LICHESS_API_BASE"));
    let url = Url::parse(&*url)?;
    let puzzle: DailyPuzzle = reqwest::get(url).await?.json::<DailyPuzzle>().await?;

    Ok(puzzle)
}

// BOARD-ENDPOINT
// privileged
pub async fn resign_game(
    game_id: &String,
    token: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/board/game/{}/resign", env!("LICHESS_API_BASE"), game_id);
    let url = Url::parse(&*url)?;
    let client = reqwest::Client::new();
    let response = client.post(url).bearer_auth(token).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to resign game: {}", response.status()).into());
    } else {
        info!("Game resigned successfully");
    }

    Ok(())
}

pub async fn move_piece(
    game_id: &String,
    board_move: &String,
    token: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/board/game/{}/move/{}",
        env!("LICHESS_API_BASE"),
        game_id,
        board_move
    );
    let url = Url::parse(&*url)?;
    let client = reqwest::Client::new();
    let response = client.post(url).bearer_auth(token).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to move piece: {}", response.status()).into());
    } else {
        info!("Piece moved successfully");
    }

    Ok(())
}
