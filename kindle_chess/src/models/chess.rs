use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;

use crate::models::board_api::BoardAPI;
use crate::models::board_local::BoardLocal;
use crate::ui::events::AppEvent;
use crate::ui::renderer::Renderer;
use crate::ui::widgets::{BoardWidget, SidebarWidget};

pub struct ChessApp {
    pub backend: ChessBackend,
    pub ui: ChessUI,
}

pub enum ChessBackend {
    Offline(BoardLocal),
    Online(BoardAPI),
}

pub struct ChessUI {
    pub renderer: Renderer,
    pub conn: Arc<x11rb::rust_connection::RustConnection>, // Using std::sync::Arc
    pub event_tx: Sender<AppEvent>,
    pub event_rx: Receiver<AppEvent>,

    // Widgets
    pub board: BoardWidget,
    pub sidebar: SidebarWidget,

    // Triple-tap detection for emergency exit
    pub tap_times: Vec<Instant>,
    pub last_tap_pos: Option<(i16, i16)>,
}
