use crate::api::models::DailyPuzzle;
use reqwest::Url;

const LICHESS_API_BASE: &str = "https://lichess.org/api";

// Test non-privileged API-Call
pub async fn get_daily_puzzle() -> Result<DailyPuzzle, Box<dyn std::error::Error>> {
    let url = format!("{}/puzzle/daily", LICHESS_API_BASE);
    let url = Url::parse(&*url)?;
    let puzzle: DailyPuzzle = reqwest::get(url).await?.json::<DailyPuzzle>().await?;

    Ok(puzzle)
}

// Test privileged API-call
pub async fn resign_game(
    game_id: &String,
    token: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/board/game/{}/resign", LICHESS_API_BASE, game_id);
    let url = Url::parse(&*url)?;
    let client = reqwest::Client::new();
    let response = client.post(url).bearer_auth(token).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to get user info: {}", response.status()).into());
    } else {
        println!("Game aborted successfully. Loser");
    }

    Ok(())
}
