use std::thread;

use log::{error, info};
use x11rb::{connection::Connection, protocol::Event as X11Event};

use crate::{
    models::{
        app::App,
        ui::{Display, HomeScreen, Screen, Transition},
    },
    ui::events::{AppEvent, TouchEvent, TouchKind},
};

impl App {
    /// Creates the App by wiring together the single Display (X11 connection +
    /// renderer) and pushing the HomeScreen as the first entry on the stack.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let display = Display::new()?;
        let home: Box<dyn Screen> = Box::new(HomeScreen::new());
        info!("Starting App Instance");

        Ok(Self {
            display,
            screen_stack: vec![home],
        })
    }

    pub fn run(mut self) {
        info!("Triple-tap anywhere to emergency exit");

        // Spawn X11 event listener with the shared connection
        self.spawn_x11_listener();

        // Initial render of whatever is on top of the stack
        if let Some(screen) = self.screen_stack.last_mut() {
            if let Err(e) = screen.render(&mut self.display) {
                error!("Initial render failed: {}", e);
                return;
            }
        }

        loop {
            // Block until an event arrives from the X11 listener thread
            let event = match self.display.event_rx.recv() {
                Ok(e) => e,
                Err(_) => {
                    // Sender (X11 thread) dropped — nothing left to process
                    info!("Event channel closed, shutting down.");
                    break;
                }
            };

            // Check global triple-tap before handing to the active screen
            if let AppEvent::Touch(ref touch) = event {
                if self.check_triple_tap(touch) {
                    info!("Triple-tap detected — emergency exit");
                    break;
                }
            }

            // Delegate to the screen on top of the stack
            let transition = match self.screen_stack.last_mut() {
                Some(screen) => match screen.handle_event(event, &mut self.display) {
                    Ok(t) => t,
                    Err(e) => {
                        error!("Screen event error: {}", e);
                        Transition::Stay
                    }
                },
                None => {
                    // Empty stack — nothing left to show
                    break;
                }
            };

            match transition {
                Transition::Stay => {}

                Transition::Redraw => {
                    if let Some(screen) = self.screen_stack.last_mut() {
                        if let Err(e) = screen.render(&mut self.display) {
                            error!("Render error: {}", e);
                        }
                    }
                }

                Transition::Push(new_screen) => {
                    self.screen_stack.push(new_screen);
                    if let Some(screen) = self.screen_stack.last_mut() {
                        if let Err(e) = screen.render(&mut self.display) {
                            error!("Render error after push: {}", e);
                        }
                    }
                }

                Transition::Pop => {
                    self.screen_stack.pop();
                    if self.screen_stack.is_empty() {
                        info!("Screen stack empty — exiting");
                        break;
                    }
                    if let Some(screen) = self.screen_stack.last_mut() {
                        if let Err(e) = screen.render(&mut self.display) {
                            error!("Render error after pop: {}", e);
                        }
                    }
                }

                Transition::Quit => {
                    info!("Quit requested");
                    break;
                }
            }
        }

        info!("App shutting down");
    }

    /// Returns true if the given touch completes a triple-tap gesture.
    /// Resets the counter whenever taps drift more than 50 px apart or the
    /// 500 ms window expires.
    fn check_triple_tap(&mut self, touch: &TouchEvent) -> bool {
        if touch.kind != TouchKind::Down {
            return false;
        }

        let now = std::time::Instant::now();
        let pos = (touch.x, touch.y);

        match self.display.last_tap_pos {
            Some(last) if (pos.0 - last.0).abs() <= 50 && (pos.1 - last.1).abs() <= 50 => {
                self.display.tap_times.push(now);
                self.display
                    .tap_times
                    .retain(|t| now.duration_since(*t) < std::time::Duration::from_millis(500));

                if self.display.tap_times.len() >= 3 {
                    return true;
                }
            }
            _ => {
                // Too far away or first tap — reset
                self.display.tap_times.clear();
                self.display.tap_times.push(now);
                self.display.last_tap_pos = Some(pos);
            }
        }

        false
    }

    fn spawn_x11_listener(&self) {
        let tx = self.display.event_tx.clone();
        let conn = self.display.conn.clone();

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

                            // X11Event::KeyPress(_) => {
                            //     info!("Hardware button pressed");
                            //     Some(AppEvent::Quit)
                            // }
                            X11Event::UnmapNotify(_) => Some(AppEvent::WindowUnmapped),

                            _ => None,
                        };

                        if let Some(ev) = app_event {
                            if tx.send(ev).is_err() {
                                break; // Main thread dropped the receiver
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
