use crate::models::board_api::BoardAPI;
use crate::models::board_local::BoardLocal;

pub struct ChessApp {
    pub backend: ChessBackend,
}

pub enum ChessBackend {
    Offline(BoardLocal),
    Online(BoardAPI),
}
