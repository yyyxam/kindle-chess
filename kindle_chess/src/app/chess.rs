/* TODO (backlog)
 * Online play or
 * Local play (to be implemented)
 * as different ChessBackends
 */

use crate::models::{
    board_api::BoardAPI,
    board_local::BoardLocal,
    chess::{
        ChessApp,
        ChessBackend::{self},
    },
    oauth::{LichessUser, TokenInfo},
};

impl ChessApp {
    /// Constructs an online backend from an already-authenticated token.
    /// Authentication must be completed before calling this.
    pub async fn new_online(
        token: TokenInfo,
        user: LichessUser,
    ) -> Result<ChessApp, Box<dyn std::error::Error>> {
        let board_api = BoardAPI::new((token, user)).await?;
        Ok(Self {
            backend: ChessBackend::Online(board_api),
        })
    }

    /// Constructs an offline backend (not implemented).
    pub fn new_offline() -> ChessApp {
        let board_local = BoardLocal::new(String::new());
        Self {
            backend: ChessBackend::Offline(board_local),
        }
    }

    // Getter for the API
    #[allow(dead_code)]
    fn board_api(&mut self) -> Option<&mut BoardAPI> {
        match &mut self.backend {
            ChessBackend::Offline(_) => None,
            ChessBackend::Online(board_api) => Some(board_api),
        }
    }

    /// Returns a cloned `BoardAPI` for use inside async tasks (BoardAPI is `Clone`,
    /// holds only the auth token + user). Returns `None` for offline backends.
    pub fn online_api(&self) -> Option<BoardAPI> {
        match &self.backend {
            ChessBackend::Online(api) => Some(api.clone()),
            ChessBackend::Offline(_) => None,
        }
    }

    // pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
    //     /* Starts the ChessGameScreen (TODO) and loads/starts a game into the backend (TODO).
    //      * For now, crudely checks if it's an online game and if so, passes game_id of most recent game to stream it
    //      */
    //     let is_online = matches!(self.backend, ChessBackend::Online(_));

    //     let on_games: GameDataList;
    //     if let Some(api) = self.board_api() {
    //         let on_games = api.get_ongoing_games(5).await.unwrap().now_playing;
    //     };
    //     for game in &on_games {
    //         info!("Retrieved game-id {}", &game.full_id);
    //     }
    //     let game_id = on_games[0].full_id.clone();
    //     info!("Streaming game id: {}", &game_id);
    //     Ok(())
    // }
}
