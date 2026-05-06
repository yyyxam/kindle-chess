use std::{
    error::Error,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::Instant,
};

use image::{ImageBuffer, Luma};

use crate::{
    models::{board_api::GameDataList, chess::ChessApp},
    ui::{
        events::{AppEvent, Rectangle, RectangleExt},
        renderer::Renderer,
        widgets::{BoardWidget, Button, SidebarWidget},
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
    pub chess_button: Button,
    pub ongoing_games_button: Button,
    // pub game_of_ur_button: Button,
    // pub settings_button: Button,

    // Auth bootstrap state. `auth_started` flips to true on the first render so
    // we kick the token check exactly once. Buttons stay inert until `app` is
    // populated — either by the bootstrap (token already valid) or by a
    // ChessReady event bubbling up from a popped ChessAuthScreen.
    pub app: Option<ChessApp>,
    pub auth_started: bool,
}

impl HomeScreen {
    pub fn new() -> Self {
        // Layout: 1072 × 1448 total canvas
        // Place the chess button centred in the upper half.
        const BTN_W: u16 = 600;
        const BTN_H: u16 = 120;
        const CENTER_X: i16 = 1072 / 2; // 336
        const CENTER_Y: i16 = 1448 / 2; // 304

        Self {
            chess_button: Button::new(
                CENTER_X - BTN_W as i16 / 2,
                CENTER_Y - (10 + BTN_H as i16),
                BTN_W,
                BTN_H,
                String::from("Demo"),
                45.0,
                true,
            ),
            ongoing_games_button: Button::new(
                CENTER_X - BTN_W as i16 / 2,
                CENTER_Y + 10 + BTN_H as i16,
                BTN_W,
                BTN_H,
                String::from("Ongoing Games"),
                45.0,
                true,
            ),
            app: None,
            auth_started: false,
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

// ─── ChessGameScreen ──────────────────────────────────────────────────────────

pub struct OngoingChessGamesScreen {
    pub app: ChessApp,
    pub prev_page_button: Button,
    pub next_page_button: Button,
    pub chessgame_button_0: Button,
    pub chessgame_button_1: Button,
    pub chessgame_button_2: Button,
    pub chessgame_button_3: Button,
    pub back_button: Button,

    // Async fetch state. `games == None && error == None && !loading` means the
    // screen has not yet kicked off its initial fetch — `render` will trigger it.
    pub games: Option<Arc<GameDataList>>,
    pub error: Option<String>,
    pub loading: bool,
}

impl OngoingChessGamesScreen {
    pub fn new(app: ChessApp) -> Self {
        // Layout: 1072 × 1448 total canvas
        // Place the chess button centred in the upper half.
        const BTN_W: u16 = 800;
        const BTN_H: u16 = 120;
        const CENTER_X: i16 = 1072 / 2; // 336
        const CENTER_Y: i16 = 1448 / 2; // 304
        Self {
            app: app,
            prev_page_button: Button::new(
                CENTER_X - (BTN_W as i16 / 2 + 8),
                CENTER_Y * 2 / 5,
                BTN_W / 2,
                BTN_H - 20,
                "<".to_string(),
                40.0,
                true,
            ),
            next_page_button: Button::new(
                CENTER_X + 8,
                CENTER_Y * 2 / 5,
                BTN_W / 2,
                BTN_H - 20,
                ">".to_string(),
                40.0,
                true,
            ),
            chessgame_button_0: Button::new(
                CENTER_X - (8 + BTN_W as i16 / 2),
                CENTER_Y - (BTN_H as i16 / 2 + 10),
                BTN_W / 2,
                BTN_H,
                "-".to_string(),
                40.0,
                true,
            ),
            chessgame_button_1: Button::new(
                CENTER_X + 8,
                CENTER_Y - (BTN_H as i16 / 2 + 10),
                BTN_W / 2,
                BTN_H,
                "-".to_string(),
                40.0,
                true,
            ),
            chessgame_button_2: Button::new(
                CENTER_X - (8 + BTN_W as i16 / 2),
                CENTER_Y + (BTN_H as i16 / 2 + 10),
                BTN_W / 2,
                BTN_H,
                "-".to_string(),
                40.0,
                true,
            ),
            chessgame_button_3: Button::new(
                CENTER_X + 8,
                CENTER_Y + (BTN_H as i16 / 2 + 10),
                BTN_W / 2,
                BTN_H,
                String::from("-"),
                40.0,
                true,
            ),
            back_button: Button::new(
                CENTER_X - BTN_W as i16 / 2,
                CENTER_Y * 2 - (BTN_H as i16 - 20) - 32,
                BTN_W,
                BTN_H - 20,
                "back".to_string(),
                40.0,
                true,
            ),

            games: None,
            error: None,
            loading: false,
        }
    }
}

// ─── ChessAuthScreen ──────────────────────────────────────────────────────────

pub struct ChessAuthScreen {
    pub qr_code: Rectangle,
    pub auth_status: Rectangle,
    pub qr_image: Option<ImageBuffer<Luma<u8>, Vec<u8>>>,
    pub auth_url: Option<String>,
    // First-render flag: kick the QR/authenticate flow exactly once.
    pub auth_started: bool,
}

impl ChessAuthScreen {
    pub fn new() -> Self {
        Self {
            qr_code: Rectangle::new(286, 400, 500, 500),
            auth_status: Rectangle::new(286, 940, 500, 60),
            qr_image: None,
            auth_url: None,
            auth_started: false,
        }
    }
}

// ─── ChessSettingsScreen ──────────────────────────────────────────────────────

pub struct ChessSettingsScreen {
    pub option_button: Rectangle,
    pub back_button: Rectangle,
}
