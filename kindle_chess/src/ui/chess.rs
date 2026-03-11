use crate::api::oauth::get_authenticated;
use crate::models::board_api::BoardAPI;
use crate::models::board_local::BoardLocal;
use crate::models::chess::{ChessBackend, ChessUI};
use crate::ui::events::{AppEvent, Rectangle, RectangleExt, TouchEvent, TouchKind};
use crate::ui::renderer::Renderer;
use crate::ui::widgets::board::BoardWidget;
use crate::ui::widgets::sidebar::SidebarWidget;
use log::{debug, error, info, warn};
use std::sync::Arc; // Standard Arc
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::protocol::Event as X11Event;

impl ChessUI {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();

        // Get both renderer and shared connection
        let (renderer, conn) = Renderer::new()?;

        // Layout: 1072x1448 total
        let board_area = Rectangle::new(0, 0, 1072, 1072);
        let sidebar_area = Rectangle::new(0, 1072, 1072, 376);

        Ok(Self {
            renderer,
            conn,
            event_tx: tx,
            event_rx: rx,
            board: BoardWidget::new(board_area),
            sidebar: SidebarWidget::new(sidebar_area),
            tap_times: Vec::new(),
            last_tap_pos: None,
        })
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Spawn X11 event listener with the shared connection
        self.spawn_x11_listener();

        // Initial render
        self.render()?;

        let mut last_tick = Instant::now();

        loop {
            // Non-blocking event receive
            match self.event_rx.recv_timeout(Duration::from_millis(16)) {
                Ok(event) => {
                    if !self.handle_event(event)? {
                        break; // Exit requested
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Send periodic tick for animations/timers
                    let now = Instant::now();
                    let delta = now.duration_since(last_tick);
                    if delta > Duration::from_millis(100) {
                        self.sidebar.increment_event_count();
                        self.render()?;
                        last_tick = now;
                    }
                }
                Err(e) => {
                    error!("Event channel error: {}", e);
                    break;
                }
            }
        }

        info!("ChessUI shutting down");
        Ok(())
    }

    fn spawn_x11_listener(&self) {
        let tx = self.event_tx.clone();
        let conn = self.conn.clone(); // Clone the Arc

        thread::spawn(move || {
            loop {
                match conn.wait_for_event() {
                    Ok(event) => {
                        let app_event = match event {
                            X11Event::Expose(_) => Some(AppEvent::Expose),

                            X11Event::ButtonPress(e) => Some(AppEvent::Touch(TouchEvent {
                                x: e.event_x,
                                y: e.event_y,
                                kind: TouchKind::Down,
                            })),

                            X11Event::ButtonRelease(e) => Some(AppEvent::Touch(TouchEvent {
                                x: e.event_x,
                                y: e.event_y,
                                kind: TouchKind::Up,
                            })),

                            X11Event::KeyPress(_) => {
                                info!("Hardware button pressed");
                                Some(AppEvent::Quit)
                            }

                            X11Event::UnmapNotify(_) => Some(AppEvent::WindowUnmapped),

                            _ => None,
                        };

                        if let Some(event) = app_event {
                            if tx.send(event).is_err() {
                                break; // Main thread gone
                            }
                        }
                    }
                    Err(e) => {
                        error!("X11 error: {:?}", e);
                        break;
                    }
                }
            }
        });
    }
}
