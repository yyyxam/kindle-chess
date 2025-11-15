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
        chess::{Chess, ChessBackend},
    },
};

impl Chess {
    pub async fn new(online: bool, game_id: String) -> Result<Chess, Box<dyn std::error::Error>> {
        let backend: ChessBackend = match online {
            true => {
                // Authenticate

                let auth = get_authenticated().await.unwrap();

                let board_api = BoardAPI::new(game_id, auth).await?;
                ChessBackend::Online(board_api)
            }
            false => {
                let board_local = BoardLocal::new(game_id).await;
                ChessBackend::Offline(board_local)
            }
        };

        // TODO: implement UI conenction and start x11 window here
        Ok(Self { backend: backend })
    }
}
