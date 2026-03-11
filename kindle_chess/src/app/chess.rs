/* TODO (backlog)
 * Online play or
 * Local play (to be implemented)
 * as different ChessBackends
 */

use crate::{
    api::oauth::get_authenticated,
    models::{
        board_api::BoardAPI,
        board_local::BoardLocal,
        chess::{ChessApp, ChessBackend, ChessUI},
    },
};

impl ChessApp {
    pub async fn new(online: bool) -> Result<ChessApp, Box<dyn std::error::Error>> {
        // INIT backend
        let backend: ChessBackend = match online {
            true => {
                // Authenticate
                // TODO: Check if authenticated or not. Enable/Disable online play accordingly
                // At this point, we should get recent games via API
                // These are to be displayed on the Chess Game Homescreen, which should be shown now
                // Then we should wait for user input, which could bring us to
                // 1. Initialize a Game
                // 1.a From ongoing games
                // 1.b Creating a new one
                // 2. Show Settings
                // ....
                // 3. Exit App
                //
                // For now, just start the most recent online game
                let auth = get_authenticated().await.unwrap();
                let mut board_api = BoardAPI::new(auth).await?;
                // board_api.stream_game_event().await.unwrap();
                ChessBackend::Online(board_api)
            }
            false => {
                let board_local = BoardLocal::new(game_id).await;
                ChessBackend::Offline(board_local)
            }
        };

        // INIT ui — only construct here, do NOT call run() yet
        let ui: ChessUI = ChessUI::new().await?;

        Ok(Self { backend, ui })
    }

    /// Starts the UI event loop. Consumes the ChessApp since ChessUI::run() consumes self.
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        self.ui.run()
    }
}
