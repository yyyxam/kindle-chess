use log::LevelFilter;
use log::info;
use log4rs::Handle;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

pub mod api {
    pub mod api;
    pub mod models;
    pub mod oauth;
}

use api::api::resign_game;
use api::oauth::get_authenticated;

// use crate::api::oauth::logout;
//use crate::api::api::get_daily_puzzle;

#[tokio::main]
async fn main() {
    init_log();
    // DAILY-PUZZLE-TEST
    // info!("Retreiving Daily Puzzle...");
    // match get_daily_puzzle().await {
    //     Ok(daily_puzzle) => {
    //         info!("Retrieved Daily Puzzle!");
    //         info!("Puzzle is {:?}", daily_puzzle.game);
    //         info!("Some stats: {:?}", daily_puzzle.puzzle);
    //     }
    //     Err(e) => {
    //         info!("Error retrieving puzzle: {}", e)
    //     }
    // }

    // let mut auth_token: String = String::new();
    // let game_id: String = String::from("LG4IZg4k");

    info!("First try of authenticating..");
    let auth_token: String = get_authenticated().await.unwrap();
    info!("The auth-token so far is: {}", auth_token);

    info!("Successfully authenticated");

    // LOGOUT / TOKEN-DELETE-TEST
    // match logout() {
    //     Ok(()) => {
    //         println!("Token deleted!")
    //     }
    //     Err(e) => {
    //         println!("Token deletion error: {}", e)
    //     }
    // }

    // info!("Trying to abort game {}", game_id);
    // match resign_game(&game_id, &auth_token).await {
    //     Ok(()) => {
    //         println!("Auth-request flow worked!");
    //     }
    //     Err(e) => {
    //         eprintln!("Auth-request failed: {}", e);
    //     }
    // }
}

fn init_log() -> Handle {
    // LOGGING
    // 1. Appender für die Datei definieren
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("/mnt/us/hellokindle/tmp/rust_app.log")
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
