use exitfailure::ExitFailure;
use log::LevelFilter;
use log::info;
use log4rs::Handle;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ChessGame {
    game: Game,
    puzzle: Puzzle,
}

#[derive(Serialize, Deserialize, Debug)]
struct Game {
    id: String,
    perf: Performance,
    rated: bool,
    players: Vec<Player>,
    pgn: String,
    clock: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Puzzle {
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

impl ChessGame {
    async fn get() -> Result<Self, ExitFailure> {
        let url = "https://lichess.org/api/puzzle/daily";
        let url = Url::parse(&*url)?;
        let res = reqwest::get(url).await?.json::<ChessGame>().await?;

        Ok(res)
    }
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    init_log();
    info!("This is a test log!");

    let res = ChessGame::get().await?;
    info!("{:?} is todays puzzle", res.game);
    Ok(())
}

fn init_log() -> Handle {
    // LOGGING
    // 1. Appender für die Datei definieren
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("output.log")
        .unwrap();

    // // 2. Logging-Konfiguration erstellen
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    // // 3. Logger initialisieren
    let logger = log4rs::init_config(config).unwrap();

    // // 4. Loslegen mit dem Logging
    info!("Anwendung gestartet. Log-Nachrichten werden in 'output.log' geschrieben.");

    logger
}
