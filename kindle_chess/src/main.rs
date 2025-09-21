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

use api::models::DailyPuzzle;
use api::oauth::start_oauth_flow;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_log();
    let client_id = "your_lichess_client_id".to_string();
    let client_secret = "your_lichess_client_secret".to_string();

    info!("Starting OAuth flow..");
    let access_token = start_oauth_flow(client_id, client_secret).await?;

    info!("Successfully authenticated");
    info!("Access token: {}", &access_token[0..10]);

    // let res_puzzle = DailyPuzzle::get().await?;
    // info!("{:?} is todays puzzle", res_puzzle.game);

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
