use crate::events::{AppEvent, ChessMove, Rectangle, Square, TouchEvent, TouchKind};
use crate::renderer::{DrawColor, Renderer};
use log::info;

const SQUARE_SIZE: u16 = 134; // 1072 / 8

pub struct BoardWidget {
    area: Rectangle,
    selected_square: Option<Square>,
    last_touch: Option<(i16, i16)>,
    flipped: bool, // View from black's perspective
}

impl BoardWidget {
    pub fn new(area: Rectangle) -> Self {
        Self {
            area,
            selected_square: None,
            last_touch: None,
            flipped: false,
        }
    }

    pub fn handle_touch(&mut self, touch: &TouchEvent) -> Option<AppEvent> {
        if !self.area.contains(touch.x, touch.y) {
            return None;
        }

        // Convert to board coordinates
        let board_x = touch.x - self.area.x;
        let board_y = touch.y - self.area.y;

        let file = (board_x / SQUARE_SIZE as i16) as u8;
        let rank = 7 - (board_y / SQUARE_SIZE as i16) as u8;

        if file > 7 || rank > 7 {
            return None;
        }

        let square = Square::new(
            if self.flipped { 7 - file } else { file },
            if self.flipped { 7 - rank } else { rank },
        );

        match touch.kind {
            TouchKind::Down => {
                info!("Board touched at {}", square.to_algebraic());
                self.last_touch = Some((touch.x, touch.y));

                if let Some(selected) = self.selected_square {
                    if selected != square {
                        // Make move
                        let chess_move = ChessMove {
                            from: selected,
                            to: square,
                        };
                        self.selected_square = None;
                        return Some(AppEvent::MoveMade(chess_move));
                    } else {
                        // Deselect
                        self.selected_square = None;
                    }
                } else {
                    // Select square
                    self.selected_square = Some(square);
                    return Some(AppEvent::SquareSelected(square));
                }
            }
            TouchKind::Up => {
                self.last_touch = None;
            }
            _ => {}
        }

        None
    }

    pub fn render(&self, renderer: &mut Renderer) -> Result<(), Box<dyn std::error::Error>> {
        // Draw board squares
        for rank in 0..8 {
            for file in 0..8 {
                let is_dark = (rank + file) % 2 == 1;
                let color = if is_dark {
                    DrawColor::DarkGray
                } else {
                    DrawColor::LightGray
                };

                let x = self.area.x + (file * SQUARE_SIZE) as i16;
                let y = self.area.y + ((7 - rank) * SQUARE_SIZE) as i16;

                renderer.draw_rectangle(
                    Rectangle::new(x, y, SQUARE_SIZE, SQUARE_SIZE),
                    color,
                    true,
                )?;
            }
        }

        // Draw board border
        renderer.draw_rectangle(self.area, DrawColor::Black, false)?;

        // Highlight selected square
        if let Some(square) = self.selected_square {
            let file = if self.flipped {
                7 - square.file
            } else {
                square.file
            };
            let rank = if self.flipped {
                7 - square.rank
            } else {
                square.rank
            };

            let x = self.area.x + (file as i16 * SQUARE_SIZE as i16);
            let y = self.area.y + ((7 - rank) as i16 * SQUARE_SIZE as i16);

            // Draw selection border
            for i in 0..3 {
                renderer.draw_rectangle(
                    Rectangle::new(
                        x + i,
                        y + i,
                        SQUARE_SIZE - (i * 2) as u16,
                        SQUARE_SIZE - (i * 2) as u16,
                    ),
                    DrawColor::Black,
                    false,
                )?;
            }
        }

        // Draw touch indicator
        if let Some((tx, ty)) = self.last_touch {
            renderer.draw_circle(tx, ty, 30, DrawColor::Gray)?;
            renderer.draw_line(tx - 40, ty, tx + 40, ty, DrawColor::Gray)?;
            renderer.draw_line(tx, ty - 40, tx, ty + 40, DrawColor::Gray)?;
        }

        Ok(())
    }
}
