/* TODO (backlog)
 * Online play or
 * Local play (to be implemented)
 * as different ChessBackends
 */

use crate::{
    api::oauth::get_authenticated,
    models::{
        board::BoardAPI,
        chess::{Chess, ChessBackend, LocalBoardAPI},
    },
};

impl Chess {
    pub async fn new(online: bool) -> Result<Chess, Box<dyn std::error::Error>> {
        let backend = match online {
            true => {
                // Authenticate

                let auth = get_authenticated().await.unwrap();

                BoardAPI::new(game_id, auth)
            }
            false => LocalBoardAPI::new,
        };
        Ok(Self { backend: backend })
    }
}
