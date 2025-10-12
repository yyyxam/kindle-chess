use crate::api::{
    models::{DailyPuzzle, GameStateStreamEvent, HttpMethod, StreamEvent},
    oauth::authenticated_request,
};
use log::{info, warn};
use reqwest::Url;

use reqwest_streams::*;

use futures::stream::StreamExt;

//EVENT-STREAM-ENDPOINT
pub async fn stream_event(token: &String) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/stream/event", env!("LICHESS_API_BASE"));

    info!("Getting event stream");
    let mut response = authenticated_request(url, &token, HttpMethod::GET)
        .await?
        .json_nl_stream::<StreamEvent>(1024);

    // Process each event as it arrives
    while let Some(result) = response.next().await {
        match result {
            Ok(event) => {
                info!("Received event: {:?}", event);
                // Handle the event based on its type
                handle_event(event).await?;
            }
            Err(e) => {
                // Ignore the stream ping (=empty line)
                if e.to_string().contains("EOF while parsing") {
                    continue;
                }
                warn!("Error parsing event: {}", e);
            }
        }
    }

    Ok(())
}

pub async fn handle_event(event: StreamEvent) -> Result<(), Box<dyn std::error::Error>> {
    match event {
        StreamEvent::GameStart(_) => {
            info!("Issa GameStartEvent")
        }
        StreamEvent::GameFinish(_) => {
            info!("Issa GameFinishEvent")
        }
        StreamEvent::Challenge(_) => {
            info!("Issa ChallengeEvent")
        }
        StreamEvent::ChallengeDeclined(_) => {
            info!("Issa ChallengeDeclinedEvent")
        }
    }
    Ok(())
}

// BOARD-STATE-STREAM-ENDPOINT
pub async fn stream_game_event(
    game_id: &String,
    token: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{}/board/game/stream/{}", env!("LICHESS_API_BASE"), game_id);

    info!("Getting game state stream");
    let mut response = authenticated_request(url, &token, HttpMethod::STREAM)
        .await?
        .json_nl_stream::<GameStateStreamEvent>(1024);
    // Process each event as it arrives
    while let Some(result) = response.next().await {
        match result {
            Ok(event) => {
                info!("Received event: {:?}", event);
                // Handle the event based on its type
                handle_game_event(event).await?;
            }
            Err(e) => {
                // Ignore the stream ping (=empty line)
                if e.to_string().contains("EOF while parsing") {
                    continue;
                }
                warn!("Error parsing event: {}", e);
            }
        }
    }

    Ok(())
}

pub async fn handle_game_event(
    event: GameStateStreamEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    match event {
        GameStateStreamEvent::GameFull(_) => {
            info!("Issa GameFullEvent")
        }
        GameStateStreamEvent::GameState(_) => {
            info!("Issa GameStateEvent")
            // Update bit-board
            // Await move
            // ...
        }
        GameStateStreamEvent::ChatLine(_) => {
            info!("Issa ChatlineEvent")
        }
        GameStateStreamEvent::OpponentGone(_) => {
            info!("Issa OpponentGoneEvent")
        }
    }
    Ok(())
}

// BOARD-ENDPOINT
// BOARD - Move
pub async fn move_piece(
    game_id: &String,
    board_move: &String,
    token: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/board/game/{}/move/{}",
        env!("LICHESS_API_BASE"),
        game_id,
        board_move
    );

    let response = authenticated_request(url, &token, HttpMethod::POST)
        .await
        .unwrap();

    if !response.status().is_success() {
        return Err(format!("Failed to move piece: {}", response.status()).into());
    } else {
        info!("Piece moved successfully");
    }

    Ok(())
}

pub async fn resign_game(
    game_id: &String,
    token: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    /*!Endpoint for resigning a game.*/
    let url = format!("{}/board/game/{}/resign", env!("LICHESS_API_BASE"), game_id);

    let response = authenticated_request(url, &token, HttpMethod::POST)
        .await
        .unwrap();

    if !response.status().is_success() {
        return Err(format!("Failed to resign game: {}", response.status()).into());
    } else {
        info!("Game resigned");
    }

    Ok(())
}

pub async fn abort_game(
    game_id: &String,
    token: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    /*!Endpoint for aborting a game.
    Params:
        game_id: &String - does things
    */
    let url = format!("{}/board/game/{}/abort", env!("LICHESS_API_BASE"), game_id);

    let response = authenticated_request(url, &token, HttpMethod::POST)
        .await
        .unwrap();

    if !response.status().is_success() {
        return Err(format!("Failed to abort game: {}", response.status()).into());
    } else {
        info!("Game aborted");
    }

    Ok(())
}

// PUZZLE-ENDPOINT
pub async fn get_daily_puzzle() -> Result<DailyPuzzle, Box<dyn std::error::Error>> {
    let url = format!("{}/puzzle/daily", env!("LICHESS_API_BASE"));
    let url = Url::parse(&*url)?;
    let puzzle: DailyPuzzle = reqwest::get(url).await?.json::<DailyPuzzle>().await?;

    Ok(puzzle)
}
