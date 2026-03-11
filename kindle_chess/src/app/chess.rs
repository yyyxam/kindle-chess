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
        chess::{
            ChessApp,
            ChessBackend::{self},
        },
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

                let auth = get_authenticated().await.unwrap();
                let board_api = BoardAPI::new(auth).await?;
                // board_api.stream_game_event().await.unwrap();
                ChessBackend::Online(board_api)
            }
            false => {
                // TODO: implement local game engine with id system and pass a proper game_id below
                let board_local = BoardLocal::new(String::new()).await;
                ChessBackend::Offline(board_local)
            }
        };

        Ok(Self { backend })
    }

    // Getter for the API
    fn board_api(&mut self) -> Option<&mut BoardAPI> {
        match &mut self.backend {
            ChessBackend::Offline(_) => None,
            ChessBackend::Online(board_api) => Some(board_api),
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
