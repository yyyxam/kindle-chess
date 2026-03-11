use std::{
    error::Error,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::Instant,
};

use crate::ui::{
    events::{AppEvent, Rectangle},
    renderer::Renderer,
    widgets::{BoardWidget, SidebarWidget},
};

// Long-lived X11 ressource
pub struct Display {
    pub renderer: Renderer,
    pub conn: Arc<x11rb::rust_connection::RustConnection>,
    pub event_tx: Sender<AppEvent>,
    pub event_rx: Receiver<AppEvent>,

    // Widgets
    pub board: BoardWidget,
    pub sidebar: SidebarWidget,

    // Triple-tap detection for emergency exit
    pub tap_times: Vec<Instant>,
    pub last_tap_pos: Option<(i16, i16)>,
}

// Bundle of widgets and logic for a specific use (e.g. homescreen / chessscreen / settingsscreen)
pub trait Screen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn Error>>;
    fn handle_event(
        &mut self,
        event: AppEvent,
        display: &mut Display,
    ) -> Result<bool, Box<dyn Error>>;
}

pub enum Transition {
    Stay,                  // keep current screen
    Push(Box<dyn Screen>), // switch to new screen
    Pop,                   // return to previous screen
    Quit,
}

pub struct HomeScreen {
    chess_button: Rectangle,
    //game_of_ur_button: Rectangle
}

pub struct ChessGameScreen {
    board: BoardWidget,
    sidebar: SidebarWidget,
}

pub struct ChessAuthScreen {
    qr_code: Rectangle,
    auth_status: Rectangle,
}

pub struct ChessSettingsScreen {
    option_button: Rectangle,
    back_button: Rectangle,
}
