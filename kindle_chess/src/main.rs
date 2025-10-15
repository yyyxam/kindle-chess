use log::LevelFilter;
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
pub mod models;

use crate::models::board::Board;

#[tokio::main]
async fn main() {
    init_log();

    let game_id = format!("g5cDqT42uLWs");

    let mut board = Board::new(game_id).await.unwrap();
    let on_games = board.get_ongoing_games(5).await.unwrap().now_playing;

    for game in &on_games {
        println!("Retrieved game-id {}", game.full_id);
    }

    board.stream_game_event().await.unwrap();

    //board.move_piece(&game_id, "f6f5").await.unwrap();
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
