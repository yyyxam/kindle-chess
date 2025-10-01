use std::str::FromStr;

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

use api::oauth::authenticate;
use serde_json::from_str;

use crate::api::api::resign_game;

//use crate::api::api::get_daily_puzzle;

#[tokio::main]
async fn main() {
    init_log();

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

    let mut auth_token: String = String::new();
    let game_id: String = String::from("LG4IZg4k");

    info!("Starting OAuth flow..");
    match authenticate().await {
        Ok((token, user)) => {
            println!("Access token: {}", token.access_token);
            println!("Authenticated as: {}", user.username);
            auth_token = token.access_token;
        }
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
        }
    }
    info!("Successfully authenticated");
    info!("Trying to abort game {}", game_id);

    match resign_game(&game_id, &auth_token).await {
        Ok(()) => {
            println!("Auth-request flow worked!");
        }
        Err(e) => {
            eprintln!("Auth-request failed: {}", e);
        }
    }

    //info!("Access token: {}", &access_token[0..10]);
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
