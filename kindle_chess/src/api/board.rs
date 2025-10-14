use std::io;

use crate::api::oauth::{authenticated_request, get_authenticated};
use crate::models::board::{Board, GameStateStreamEvent, StreamEvent};
use crate::models::oauth::HttpMethod;
use futures::StreamExt;
use log::{info, warn};
use reqwest_streams::JsonStreamResponse;

impl Board {
    pub async fn new(game_id: String) -> Result<Board, Box<dyn std::error::Error>> {
        Ok(Self {
            token: get_authenticated().await?,
            bitboard: Vec::new(),
            game_id: game_id,
        })
    }

    pub async fn move_piece(
        &self,
        game_id: &String,
        board_move: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/board/game/{}/move/{}",
            env!("LICHESS_API_BASE"),
            game_id,
            board_move
        );

        let response = authenticated_request(url, &self.token, HttpMethod::POST)
            .await
            .unwrap();

        if !response.status().is_success() {
            return Err(format!("Failed to move piece: {}", response.status()).into());
        } else {
            info!("Piece moved successfully");
        }

        Ok(())
    }

    pub async fn resign_game(&self, game_id: &String) -> Result<(), Box<dyn std::error::Error>> {
        /*!Endpoint for resigning a game.*/
        let url = format!("{}/board/game/{}/resign", env!("LICHESS_API_BASE"), game_id);

        let response = authenticated_request(url, &self.token, HttpMethod::POST)
            .await
            .unwrap();

        if !response.status().is_success() {
            return Err(format!("Failed to resign game: {}", response.status()).into());
        } else {
            info!("Game resigned");
        }

        Ok(())
    }

    pub async fn abort_game(&self, game_id: &String) -> Result<(), Box<dyn std::error::Error>> {
        /*!Endpoint for aborting a game.
        Params:
            game_id: &String - does things
        */
        let url = format!("{}/board/game/{}/abort", env!("LICHESS_API_BASE"), game_id);

        let response = authenticated_request(url, &self.token, HttpMethod::POST)
            .await
            .unwrap();

        if !response.status().is_success() {
            return Err(format!("Failed to abort game: {}", response.status()).into());
        } else {
            info!("Game aborted");
        }

        Ok(())
    }

    // BOARD-STATE-STREAM-ENDPOINT
    pub async fn stream_game_event(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/board/game/stream/{}",
            env!("LICHESS_API_BASE"),
            self.game_id
        );

        info!("Getting game state stream");
        let mut response = authenticated_request(url, &self.token, HttpMethod::STREAM)
            .await?
            .json_nl_stream::<GameStateStreamEvent>(1024);
        // Process each event as it arrives
        while let Some(result) = response.next().await {
            match result {
                Ok(event) => {
                    info!("Received event: {:?}", event);
                    println!("Received event: {:?}", event);
                    // Handle the event based on its type
                    self.handle_game_event(event).await?;
                    // TODO: fix GameFullEvent not being parsed (=going to Err-arm)
                }
                Err(e) => {
                    // Ignore the stream ping (=empty line)
                    if e.to_string().contains("EOF while parsing") {
                        continue;
                    }
                    warn!("Error parsing event: {}", e);
                    println!("Error parsing event: {}", e);
                }
            }
        }

        Ok(())
    }

    pub async fn handle_game_event(
        &self,
        event: GameStateStreamEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            GameStateStreamEvent::GameFull(_) => {
                info!("Issa GameFullEvent")
            }
            GameStateStreamEvent::GameState(_) => {
                info!("Issa GameStateEvent");
                // Update bit-board
                // ...
                // Check if move is awaited or opponents turn
                // ...
                // Await move
                // TODO: check if response contains opps move or if it's player's move
                // (1) -> player's turn now, await input: ..
                // (2) -> do nothing and await next response
                let mut buffer = String::new();
                println!("Enter move to play!");
                // TODO implement regex to sanitize
                let reader = io::stdin();
                match reader.read_line(&mut buffer) {
                    Ok(b) => {
                        self.move_piece(&self.game_id, &buffer).await.unwrap();
                    }
                    Err(e) => {
                        println!("Failed to parse input: {}", e)
                    }
                }

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

    //EVENT-STREAM-ENDPOINT
    pub async fn stream_event(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/stream/event", env!("LICHESS_API_BASE"));

        info!("Getting event stream");
        let mut response = authenticated_request(url, &self.token, HttpMethod::GET)
            .await?
            .json_nl_stream::<StreamEvent>(1024);

        // Process each event as it arrives
        while let Some(result) = response.next().await {
            match result {
                Ok(event) => {
                    info!("Received event: {:?}", event);
                    // Handle the event based on its type
                    self.handle_event(event).await?;
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

    pub async fn handle_event(&self, event: StreamEvent) -> Result<(), Box<dyn std::error::Error>> {
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
}
