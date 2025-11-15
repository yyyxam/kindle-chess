use crate::models::board_api::BoardAPI;
use crate::models::board_local::BoardLocal;

pub struct Chess {
    pub backend: ChessBackend,
}

pub enum ChessBackend {
    Offline(BoardLocal),
    Online(BoardAPI),
}
