use crate::models::puzzle::DailyPuzzle;

use reqwest::Url;

// ~~~~~~~~~~~~~~~~ PUZZLE-ENDPOINT ~~~~~~~~~~~~~~~~
pub async fn get_daily_puzzle() -> Result<DailyPuzzle, Box<dyn std::error::Error>> {
    let url = format!("{}/puzzle/daily", env!("LICHESS_API_BASE"));
    let url = Url::parse(&*url)?;
    let puzzle: DailyPuzzle = reqwest::get(url).await?.json::<DailyPuzzle>().await?;

    Ok(puzzle)
}
