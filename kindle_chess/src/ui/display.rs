use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

use crate::{
    models::ui::Display,
    ui::{
        events::{TouchEvent, TouchKind},
        renderer::Renderer,
        widgets::{BoardWidget, SidebarWidget},
    },
};

impl Display {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
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

    pub fn check_triple_tap(&mut self, touch: &TouchEvent) -> bool {
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
}
