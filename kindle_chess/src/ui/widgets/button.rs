use crate::ui::renderer::DrawColor;
use crate::ui::{
    events::{Rectangle, RectangleExt},
    renderer::Renderer,
};

pub struct Button {
    pub rect: Rectangle,
    pub label: String,
    pub font_size: f32,
    pub outline: bool,
}

impl Button {
    pub fn new(
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        label: String,
        font_size: f32,
        outline: bool,
    ) -> Self {
        Self {
            rect: Rectangle::new(x, y, width, height),
            label,
            font_size,
            outline,
        }
    }
    pub fn draw(&self, renderer: &mut Renderer) -> Result<(), Box<dyn std::error::Error>> {
        renderer.draw_rectangle(self.rect, DrawColor::White, true)?;
        if self.outline {
            renderer.draw_rectangle(self.rect, DrawColor::Black, false)?;
        }
        let label = self.label.as_str();
        let (tw, th) = renderer.measure_text(label, self.font_size);
        let tx = self.rect.x + (self.rect.width as i16 - tw as i16) / 2;
        let ty = self.rect.y + (self.rect.height as i16 - th as i16) / 2;
        renderer.draw_text(tx, ty, label, self.font_size, DrawColor::Black)?;
        Ok(())
    }
}
