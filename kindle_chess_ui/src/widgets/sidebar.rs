use crate::events::{AppEvent, Rectangle, TouchEvent, TouchKind};
use crate::renderer::{DrawColor, Renderer};
use log::info;

pub struct SidebarWidget {
    area: Rectangle,
    menu_button: Rectangle,
    exit_button: Rectangle,
    event_count: u32,
}

impl SidebarWidget {
    pub fn new(area: Rectangle) -> Self {
        Self {
            area,
            menu_button: Rectangle::new(area.x + 10, area.y + 10, 200, 60),
            exit_button: Rectangle::new(area.x + 10, area.y + 280, 200, 60),
            event_count: 0,
        }
    }

    pub fn increment_event_count(&mut self) {
        self.event_count += 1;
    }

    pub fn handle_touch(&mut self, touch: &TouchEvent) -> Option<AppEvent> {
        if !self.area.contains(touch.x, touch.y) {
            return None;
        }

        if touch.kind == TouchKind::Up {
            if self.menu_button.contains(touch.x, touch.y) {
                info!("Menu button pressed");
                return Some(AppEvent::ShowMenu);
            }

            if self.exit_button.contains(touch.x, touch.y) {
                info!("Exit button pressed");
                return Some(AppEvent::Quit);
            }
        }

        None
    }

    pub fn render(&self, renderer: &mut Renderer) -> Result<(), Box<dyn std::error::Error>> {
        // Clear sidebar area
        renderer.draw_rectangle(self.area, DrawColor::White, true)?;

        // Draw border
        renderer.draw_rectangle(self.area, DrawColor::Black, false)?;

        // Draw menu button
        renderer.draw_rectangle(self.menu_button, DrawColor::LightGray, true)?;
        renderer.draw_rectangle(self.menu_button, DrawColor::Black, false)?;

        // Draw exit button with striped pattern
        for i in 0..3 {
            renderer.draw_rectangle(
                Rectangle::new(
                    self.exit_button.x,
                    self.exit_button.y + (i * 20),
                    self.exit_button.width,
                    10,
                ),
                DrawColor::DarkGray,
                true,
            )?;
        }
        renderer.draw_rectangle(self.exit_button, DrawColor::Black, false)?;

        // Draw event counter visualization
        let count = (self.event_count % 10) as usize;
        for i in 0..count {
            renderer.draw_rectangle(
                Rectangle::new(
                    self.area.x + 10 + (i * 25) as i16,
                    self.area.y + 100,
                    20,
                    20,
                ),
                DrawColor::Black,
                true,
            )?;
        }

        Ok(())
    }
}
