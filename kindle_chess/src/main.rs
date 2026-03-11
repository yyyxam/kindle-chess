use log::LevelFilter;
use log::error;
use log::info;
use log4rs::Handle;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

pub mod api {
    pub mod api;
    pub mod board;
    pub mod oauth;
}
pub mod app;
pub mod local;
pub mod models;
pub mod ui;

use crate::api::board::get_ongoing_games;
use crate::api::oauth::get_authenticated;
use crate::models::chess::ChessApp;

#[tokio::main]
async fn main() {
    init_log();

    // let mut board = BoardAPI::new(game_id, auth).await.unwrap();
    // board.stream_game_event().await.unwrap();
    //
    // Run the app
    match ChessApp::new(true).await {
        Ok(app) => {
            if let Err(e) = app.run() {
                error!("Application error: {}", e);
            }
        }
        Err(e) => {
            error!("Failed to initialize ChessApp: {}", e);
        }
    }
}

fn init_log() -> Handle {
    // LOGGING
    // 1. Appender für die Datei definieren
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(concat!(env!("LOG_FILE_DIR"), "app.log"))
        .unwrap();

    // // 2. Logging-Konfiguration erstellen
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    // // 3. Logger initialisieren
    let logger = log4rs::init_config(config).unwrap();

    // // 4. Loslegen mit dem Logging
    info!("Anwendung gestartet. Log-Nachrichten werden in 'app.log' geschrieben.");

    logger
}
