use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

use log::info;

use crate::{
    models::ui::Display,
    ui::events::{TouchEvent, TouchKind},
};

impl Display {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (event_tx, event_rx) = mpsc::channel();
        let (renderer, conn) = crate::ui::renderer::Renderer::new()?;
        info!("Starting Display isntance");

        Ok(Self {
            renderer,
            conn,
            event_tx,
            event_rx,
            tap_times: Vec::new(),
            last_tap_pos: None,
        })
    }

    pub fn check_triple_tap(&mut self, touch: &TouchEvent) -> bool {
        if touch.kind != TouchKind::Down {
            return false;
        }

        let now = Instant::now();
        let tap_pos = (touch.x, touch.y);

        if let Some(last_pos) = self.last_tap_pos {
            let dx = (tap_pos.0 - last_pos.0).abs();
            let dy = (tap_pos.1 - last_pos.1).abs();

            if dx <= 50 && dy <= 50 {
                self.tap_times.push(now);
                self.tap_times
                    .retain(|t| now.duration_since(*t) < Duration::from_millis(500));

                if self.tap_times.len() >= 3 {
                    return true;
                }
            } else {
                self.tap_times.clear();
                self.tap_times.push(now);
                self.last_tap_pos = Some(tap_pos);
            }
        } else {
            self.tap_times.clear();
            self.tap_times.push(now);
            self.last_tap_pos = Some(tap_pos);
        }

        false
    }
}
