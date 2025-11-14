use crate::events::{AppEvent, Rectangle, TouchEvent, TouchKind};
use crate::renderer::Renderer;
use crate::widgets::board::BoardWidget;
use crate::widgets::sidebar::SidebarWidget;
use log::{debug, error, info, warn};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc; // Standard Arc
use std::thread;
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::protocol::Event as X11Event;

pub struct ChessApp {
    renderer: Renderer,
    conn: Arc<x11rb::rust_connection::RustConnection>, // Using std::sync::Arc
    event_tx: Sender<AppEvent>,
    event_rx: Receiver<AppEvent>,

    // Widgets
    board: BoardWidget,
    sidebar: SidebarWidget,

    // Triple-tap detection for emergency exit
    tap_times: Vec<Instant>,
    last_tap_pos: Option<(i16, i16)>,
}

impl ChessApp {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
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
        info!("Starting Chess UI");
        info!("Touch chess board to select/move pieces");
        info!("Touch bottom area buttons to navigate");
        info!("Triple-tap anywhere to emergency exit");

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

        info!("Application shutting down");
        Ok(())
    }

    fn handle_event(&mut self, event: AppEvent) -> Result<bool, Box<dyn std::error::Error>> {
        match event {
            AppEvent::Touch(touch) => {
                // Check triple-tap for emergency exit
                if self.check_triple_tap(&touch) {
                    info!("Triple-tap detected - emergency exit!");
                    return Ok(false);
                }

                // Handle touch in widgets
                if let Some(event) = self.board.handle_touch(&touch) {
                    return self.handle_event(event);
                }

                if let Some(event) = self.sidebar.handle_touch(&touch) {
                    return self.handle_event(event);
                }

                self.render()?;
            }

            AppEvent::MoveMade(chess_move) => {
                info!(
                    "Move: {} -> {}",
                    chess_move.from.to_algebraic(),
                    chess_move.to.to_algebraic()
                );
                self.render()?;
            }

            AppEvent::SquareSelected(square) => {
                info!("Selected square: {}", square.to_algebraic());
                self.render()?;
            }

            AppEvent::ShowMenu => {
                info!("Menu requested");
                // TODO: Show menu overlay
            }

            AppEvent::Expose => {
                debug!("Expose event - redrawing");
                self.render()?;
            }

            AppEvent::WindowUnmapped => {
                warn!("Window unmapped!");
                // Try to remap?
            }

            AppEvent::Quit => {
                info!("Quit requested");
                return Ok(false);
            }

            _ => {}
        }

        Ok(true)
    }

    fn check_triple_tap(&mut self, touch: &TouchEvent) -> bool {
        if touch.kind != TouchKind::Down {
            return false;
        }

        let now = Instant::now();
        let tap_pos = (touch.x, touch.y);

        // Check if close to last tap
        if let Some(last_pos) = self.last_tap_pos {
            let dx = (tap_pos.0 - last_pos.0).abs();
            let dy = (tap_pos.1 - last_pos.1).abs();

            if dx <= 50 && dy <= 50 {
                self.tap_times.push(now);

                // Remove old taps
                self.tap_times
                    .retain(|t| now.duration_since(*t) < Duration::from_millis(500));

                if self.tap_times.len() >= 3 {
                    return true;
                }
            } else {
                // Too far, reset
                self.tap_times.clear();
                self.tap_times.push(now);
                self.last_tap_pos = Some(tap_pos);
            }
        } else {
            // First tap
            self.tap_times.clear();
            self.tap_times.push(now);
            self.last_tap_pos = Some(tap_pos);
        }

        false
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.board.render(&mut self.renderer)?;
        self.sidebar.render(&mut self.renderer)?;
        self.renderer.present()?;
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
