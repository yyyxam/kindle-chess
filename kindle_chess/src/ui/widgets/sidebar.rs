use crate::models::board_api::Turn;
use crate::ui::events::{AppEvent, Rectangle, RectangleExt, TouchEvent, TouchKind};
use crate::ui::renderer::{DrawColor, Renderer};
use crate::ui::widgets::Button;
use log::info;

pub struct SidebarWidget {
    area: Rectangle,
    menu_button: Button,
    back_button: Button,
    event_count: u32,
    // Driven by `set_turn` from the game-state stream events arriving on
    // ChessGameScreen. Read by `render` to draw the status line.
    turn_status: String,
}

impl SidebarWidget {
    pub fn new(area: Rectangle) -> Self {
        let button_width = 200 as i16;
        let button_height = 75 as i16;
        Self {
            area,
            back_button: Button::new(
                area.x + area.width as i16 / 2 - button_width / 2,
                area.y + area.height as i16 - (button_height + 20),
                button_width as u16,
                button_height as u16,
                "back".to_string(),
                40.0,
                true,
            ),
            menu_button: Button::new(
                area.x + area.width as i16 / 2 - button_width / 2,
                area.y + area.height as i16 - 2 * (button_height + 15),
                button_width as u16,
                button_height as u16,
                "menu".to_string(),
                40.0,
                true,
            ),
            event_count: 0,
            turn_status: String::from("Loading…"),
        }
    }

    pub fn increment_event_count(&mut self) {
        self.event_count += 1;
    }

    pub fn set_turn(&mut self, turn: Turn) {
        self.turn_status = match turn {
            Turn::Playing => "Your turn".to_string(),
            Turn::Waiting => "Waiting for opponent".to_string(),
            Turn::Over { winner: Some(w) } => format!("Game over — winner: {}", w),
            Turn::Over { winner: None } => "Game over".to_string(),
        };
    }

    pub fn handle_touch(&mut self, touch: &TouchEvent) -> Option<AppEvent> {
        if !self.area.contains(touch.x, touch.y) {
            return None;
        }

        if touch.kind == TouchKind::Up {
            if self.menu_button.rect.contains(touch.x, touch.y) {
                info!("Menu button pressed");
                return Some(AppEvent::ShowMenu);
            }

            if self.back_button.rect.contains(touch.x, touch.y) {
                info!("Back button pressed");
                return Some(AppEvent::ExitToMenu);
            }
        }

        None
    }

    pub fn render(&self, renderer: &mut Renderer) -> Result<(), Box<dyn std::error::Error>> {
        // Clear sidebar area
        renderer.draw_rectangle(self.area, DrawColor::White, true)?;

        // Draw border
        renderer.draw_rectangle(self.area, DrawColor::Black, false)?;

        // Turn status text, centred near the top of the sidebar.
        let size_px = 32.0;
        let (tw, _th) = renderer.measure_text(&self.turn_status, size_px);
        let tx = self.area.x + (self.area.width as i16 - tw as i16) / 2;
        let ty = self.area.y + 50;
        renderer.draw_text(tx, ty, &self.turn_status, size_px, DrawColor::Black)?;

        // Draw buttons
        self.menu_button.draw(renderer)?;
        self.back_button.draw(renderer)?;

        Ok(())
    }
}
