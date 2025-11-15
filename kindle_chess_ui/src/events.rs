use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AppEvent {
    // UI Events
    Touch(TouchEvent),
    Redraw,
    Tick(Duration),

    // Chess Events
    MoveMade(ChessMove),
    SquareSelected(Square),

    // Navigation
    ShowMenu,
    ExitToMenu,
    Quit,

    // X11 Events
    Expose,
    WindowUnmapped,
}

#[derive(Debug, Clone, Copy)]
pub struct TouchEvent {
    pub x: i16,
    pub y: i16,
    pub kind: TouchKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchKind {
    Down,
    Up,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Square {
    pub file: u8, // 0-7 (a-h)
    pub rank: u8, // 0-7 (1-8)
}

impl Square {
    pub fn new(file: u8, rank: u8) -> Self {
        Self { file, rank }
    }

    pub fn to_algebraic(&self) -> String {
        format!("{}{}", (b'a' + self.file) as char, self.rank + 1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChessMove {
    pub from: Square,
    pub to: Square,
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl Rectangle {
    pub fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, px: i16, py: i16) -> bool {
        px >= self.x
            && px < self.x + self.width as i16
            && py >= self.y
            && py < self.y + self.height as i16
    }
}
