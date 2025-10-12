// DAILY-PUZZLE-TEST
// info!("Retreiving Daily Puzzle...");
// match get_daily_puzzle().await {
//     Ok(daily_puzzle) => {
//         info!("Retrieved Daily Puzzle!");
//         info!("Puzzle is {:?}", daily_puzzle.game);
//         info!("Some stats: {:?}", daily_puzzle.puzzle);
//     }
//     Err(e) => {
//         info!("Error retrieving puzzle: {}", e)
//     }
// }

// let mut auth_token: String = String::new();
// let game_id: String = String::from("LG4IZg4k");

// LOGOUT / TOKEN-DELETE-TEST
// match logout() {
//     Ok(()) => {
//         println!("Token deleted!")
//     }
//     Err(e) => {
//         println!("Token deletion error: {}", e)
//     }
// }

// BOARD-INTERACTION
// info!("Trying to abort game {}", game_id);
// match resign_game(&game_id, &auth_token).await {
//     Ok(()) => {
//         println!("Auth-request flow worked!");
//     }
//     Err(e) => {
//         eprintln!("Auth-request failed: {}", e);
//     }
// }

// move_piece(&game_id, &board_move, &auth_token)
//     .await
//     .unwrap();

// resign_game(&game_id, &auth_token).await.unwrap();
