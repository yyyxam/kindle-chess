use crate::ChessGame;
use exitfailure::ExitFailure;
use reqwest::Url;

impl ChessGame {
    pub async fn get() -> Result<Self, ExitFailure> {
        let url = "https://lichess.org/api/puzzle/daily";
        let url = Url::parse(&*url)?;
        let res = reqwest::get(url).await?.json::<ChessGame>().await?;

        Ok(res)
    }
}
