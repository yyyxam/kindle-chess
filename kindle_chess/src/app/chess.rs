/* TODO (backlog)
 * Online play or
 * Local play (to be implemented)
 * as different ChessBackends
 */

use crate::{
    api::oauth::get_authenticated,
    models::{board_api::BoardAPI, board_local::BoardLocal, chess::Chess},
};

impl Chess {
    pub async fn new(online: bool, game_id: String) -> Result<Chess, Box<dyn std::error::Error>> {
        let backend = match online {
            true => {
                // Authenticate

                let auth = get_authenticated().await.unwrap();

                BoardAPI::new(game_id, auth)
            }
            false => BoardLocal::new(game_id),
        };

        // TODO: implement UI conenction and start x11 window here
        Ok(Self { backend: backend })
    }
}
