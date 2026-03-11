use log::{debug, info, warn};

use crate::{
    models::ui::{Display, HomeScreen, Screen},
    ui::events::AppEvent,
};

impl Screen for HomeScreen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn std::error::Error>> {
        display.board.render(&mut display.renderer)?;
        display.sidebar.render(&mut display.renderer)?;
        display.renderer.present()?;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: AppEvent,
        display: &mut Display,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match event {
            AppEvent::Touch(touch) => {
                // Check triple-tap for emergency exit
                if display.check_triple_tap(&touch) {
                    info!("Triple-tap detected - emergency exit!");
                    return Ok(false);
                }

                // Handle touch in widgets
                if let Some(event) = display.board.handle_touch(&touch) {
                    return self.handle_event(event, display);
                }

                if let Some(event) = display.sidebar.handle_touch(&touch) {
                    return self.handle_event(event, display);
                }

                self.render(display)?;
            }

            AppEvent::MoveMade(chess_move) => {
                info!(
                    "Move: {} -> {}",
                    chess_move.from.to_algebraic(),
                    chess_move.to.to_algebraic() // TODO send move to backend
                );
                self.render(display)?;
            }

            AppEvent::SquareSelected(square) => {
                info!("Selected square: {}", square.to_algebraic());
                self.render(display)?;
            }

            AppEvent::ShowMenu => {
                info!("Menu requested");
                // TODO: Show menu overlay
            }

            AppEvent::Expose => {
                debug!("Expose event - redrawing");
                self.render(display)?;
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
}
// impl Screen for ChessGameScreen { ... }
