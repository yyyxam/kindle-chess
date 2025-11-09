use crate::models::board::BoardAPI;

pub struct Chess {
    pub backend: ChessBackend,
}

pub struct LocalBoardAPI {}

pub enum ChessBackend {
    Offline(LocalBoardAPI),
    Online(BoardAPI),
}
