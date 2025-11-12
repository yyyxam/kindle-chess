<<<<<<< HEAD
use log::LevelFilter;
use log::info;
use log4rs::Handle;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

fn main() {
    init_log();
    println!("Hello, world!");
    info!("Hello World");
}

=======
use core::convert::TryInto;
use embedded_graphics::{
    pixelcolor::{Gray8, GrayColor},
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use log::{LevelFilter, info};
use log4rs::{
    Config, Handle,
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};

pub mod display {
    pub mod kindle_display;
}

use display::kindle_display::{KindleDisplay, RefreshMode};

const FB_WIDTH: usize = 1072;

const FB_HEIGHT: usize = 1448;

fn main() {
    let mut display = KindleDisplay::new_test();

    // Draw a circle with top-left at `(22, 22)` with a diameter of `20` and a white stroke
    let circle = Circle::new(Point::new(22, 22), 20)
        .into_styled(PrimitiveStyle::with_stroke(Gray8::WHITE, 1));

    circle.draw(&mut display);

    // Update the display
    display.flush(RefreshMode::Full).unwrap();
}
// LOGS
>>>>>>> 5105401 (uncomplete KidnleDisplay implementation)
fn init_log() -> Handle {
    // LOGGING
    // 1. Appender für die Datei definieren
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
<<<<<<< HEAD
        .build(concat!(env!("LOG_FILE_DIR"), "display.log"))
=======
        .build(concat!("./logs/display.log"))
>>>>>>> 5105401 (uncomplete KidnleDisplay implementation)
        .unwrap();

    // // 2. Logging-Konfiguration erstellen
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    // // 3. Logger initialisieren
    let logger = log4rs::init_config(config).unwrap();

    // // 4. Loslegen mit dem Logging
    info!("Anwendung gestartet. Log-Nachrichten werden in 'display.log' geschrieben.");

    logger
}
