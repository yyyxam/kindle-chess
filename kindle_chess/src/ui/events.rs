use image::{ImageBuffer, Luma};
use std::sync::Arc;
use std::time::Duration;
use x11rb::protocol::xproto;

use crate::models::{
    board_api::GameDataList,
    chess::ChessApp,
    oauth::{LichessUser, TokenInfo},
};

#[derive(Debug, Clone)]
pub enum AppEvent {
    // Authentication Events
    AuthSuccess(TokenInfo, LichessUser),
    AuthFailed(String),
    QrReady(ImageBuffer<Luma<u8>, Vec<u8>>),

    // Ongoing-games fetch
    OngoingGamesLoaded(Arc<GameDataList>),
    OngoingGamesFailed(String),

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
    ChessReady(ChessApp),
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

pub type Rectangle = xproto::Rectangle;

pub trait RectangleExt {
    fn new(x: i16, y: i16, width: u16, height: u16) -> Self;
    fn contains(&self, px: i16, py: i16) -> bool;
}

impl RectangleExt for Rectangle {
    fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn contains(&self, px: i16, py: i16) -> bool {
        px >= self.x
            && px < self.x + self.width as i16
            && py >= self.y
            && py < self.y + self.height as i16
    }
}
