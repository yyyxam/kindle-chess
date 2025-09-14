use crate::DailyPuzzle;
use exitfailure::ExitFailure;
use reqwest::Url;

impl DailyPuzzle {
    pub async fn get() -> Result<Self, ExitFailure> {
        let url = "https://lichess.org/api/puzzle/daily";
        let url = Url::parse(&*url)?;
        let res = reqwest::get(url).await?.json::<DailyPuzzle>().await?;

        Ok(res)
    }
}
