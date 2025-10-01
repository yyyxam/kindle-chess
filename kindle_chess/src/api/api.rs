use crate::api::models::DailyPuzzle;
use reqwest::Url;

pub async fn get_daily_puzzle() -> Result<DailyPuzzle, Box<dyn std::error::Error>> {
    let url = "https://lichess.org/api/puzzle/daily";
    let url = Url::parse(&*url)?;
    let puzzle: DailyPuzzle = reqwest::get(url).await?.json::<DailyPuzzle>().await?;

    Ok(puzzle)
}
