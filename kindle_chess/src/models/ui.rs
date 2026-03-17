use std::{
    error::Error,
    sync::{
        Arc,
        mpsc::{self, Receiver, Sender},
    },
    time::Instant,
};

use crate::{
    models::chess::ChessApp,
    ui::{
        events::{AppEvent, Rectangle, RectangleExt},
        renderer::Renderer,
        widgets::{BoardWidget, SidebarWidget},
    },
};

// ─── Display ──────────────────────────────────────────────────────────────────
// The single long-lived X11 resource. Created once in main() and borrowed by
// every Screen implementation for drawing and event routing.

pub struct Display {
    pub renderer: Renderer,
    pub conn: Arc<x11rb::rust_connection::RustConnection>,
    pub event_tx: Sender<AppEvent>,
    pub event_rx: Receiver<AppEvent>,

    // Triple-tap detection lives here because it is global (works on any screen)
    pub tap_times: Vec<Instant>,
    pub last_tap_pos: Option<(i16, i16)>,
}

// ─── Screen ───────────────────────────────────────────────────────────────────
// A Screen owns only widgets and screen-local state.
// It borrows Display for drawing and returns a Transition to drive navigation.

pub trait Screen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn Error>>;
    fn handle_event(
        &mut self,
        event: AppEvent,
        display: &mut Display,
    ) -> Result<Transition, Box<dyn Error>>;
}

// ─── Transition ───────────────────────────────────────────────────────────────

pub enum Transition {
    Stay,                  // keep current screen, no redraw needed
    Redraw,                // keep current screen, request a redraw
    Push(Box<dyn Screen>), // navigate forward to a new screen
    Pop,                   // return to the previous screen
    Quit,                  // exit the application
}

// ─── HomeScreen ───────────────────────────────────────────────────────────────
// The top-level launcher. Add a button here for every future game.

pub struct HomeScreen {
    pub chess_button: Rectangle,
    // pub game_of_ur_button: Rectangle,
}

impl HomeScreen {
    pub fn new() -> Self {
        // Layout: 1072 × 1448 total canvas
        // Place the chess button centred in the upper half.
        const BTN_W: u16 = 400;
        const BTN_H: u16 = 120;
        const CENTER_X: i16 = (1072 - BTN_W as i16) / 2; // 336
        const CENTER_Y: i16 = (1448 / 2 - BTN_H as i16) / 2; // 304

        Self {
            chess_button: Rectangle::new(CENTER_X, CENTER_Y, BTN_W, BTN_H),
        }
    }
}

// ─── ChessGameScreen ──────────────────────────────────────────────────────────

pub struct ChessGameScreen {
    pub app: ChessApp,
    pub board: BoardWidget,
    pub sidebar: SidebarWidget,
}

impl ChessGameScreen {
    pub fn new(app: ChessApp) -> Self {
        Self {
            app: app,
            board: BoardWidget::new(Rectangle::new(0, 0, 1072, 1072)),
            sidebar: SidebarWidget::new(Rectangle::new(0, 1072, 1072, 376)),
        }
    }
}

// ─── ChessAuthScreen ──────────────────────────────────────────────────────────

pub struct ChessAuthScreen {
    pub qr_code: Rectangle,
    pub auth_status: Rectangle,
}

impl ChessAuthScreen {
    pub fn new() -> Self {
        Self {
            qr_code: Rectangle::new(286, 400, 500, 500),
            auth_status: Rectangle::new(286, 940, 500, 60),
        }
    }
}

// ─── ChessSettingsScreen ──────────────────────────────────────────────────────

pub struct ChessSettingsScreen {
    pub option_button: Rectangle,
    pub back_button: Rectangle,
}
