use std::sync::mpsc;

use log::info;

use crate::models::ui::Display;

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
}
