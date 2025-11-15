use crate::models::board_local::BoardLocal;

impl BoardLocal {
    pub async fn new(game_id: String) -> Result<Chess, Box<dyn std::error::Error>> {
        println(
            "Would start local game {}... If it were implemented",
            game_id,
        )
    }
}
