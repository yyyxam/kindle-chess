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

    info!("Starting OAuth flow..");
    match authenticate().await {
        Ok((token, user)) => {
            println!("Access token: {}", token.access_token);
            println!("Authenticated as: {}", user.username);
        }
        Err(e) => {
            eprintln!("Authentication failed: {}", e);
        }
    }
    info!("Successfully authenticated");
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
