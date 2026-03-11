use std::thread;

use log::{error, info};
use serde::ser::Error;
use x11rb::protocol::Event;

use crate::{
    models::{app::App, ui::Display},
    ui::events::{AppEvent, TouchKind},
};

impl App {
    pub fn run(mut self) {
        info!("Triple-tap anywhere to emergency exit");
        // Spawn X11 event listener with the shared connection
        self.spawn_x11_listener();
        // Initial render
        // self.render()?;

        while let Some(screen) = self.screen_stack.last_mut() {
            // do stuff
            // // receive event, call screen.handle_event(), act on Transition
        }
    }

    fn spawn_x11_listener(&self) {
        let tx = self.display.event_tx.clone();
        let conn = self.display.conn.clone(); // Clone the Arc

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
