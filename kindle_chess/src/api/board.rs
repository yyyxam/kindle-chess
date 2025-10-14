use crate::api::oauth::{authenticated_request, get_authenticated};
use crate::app::game::{get_turn_input, player0_turn};
use crate::models::board::{Board, GameStateStreamEvent, PlayedBy, StreamEvent};
use crate::models::oauth::HttpMethod;
use futures::StreamExt;
use log::{info, warn};
use reqwest_streams::JsonStreamResponse;

impl Board {
    pub async fn new(game_id: String) -> Result<Board, Box<dyn std::error::Error>> {
        let (token, user) = get_authenticated().await?;
        Ok(Self {
            token: token,
            user: user,
            bitboard: Vec::new(),
            game_id: game_id,
            // These should all get updated with the start of the game-state-stream
            white: None,
            black: None,
            player0_white: false,
            player0_turn: false,
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
            info!("Failed to move piece");
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
    pub async fn stream_game_event(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        /*!
         * Streams ndjson-Responses.
         * = New line = new event;
         * Tries to parse from stream to EventTypes.
         */
        let url = format!(
            "{}/board/game/stream/{}",
            env!("LICHESS_API_BASE"),
            self.game_id
        );

        info!("Game-State-Stream started..");

        let mut response = authenticated_request(url, &self.token, HttpMethod::STREAM)
            .await?
            .json_nl_stream::<GameStateStreamEvent>(1024);

        // Process each event as it arrives
        while let Some(result) = response.next().await {
            match result {
                Ok(event) => {
                    info!("Received event: {:?}", event);
                    // Handle the event based on its type
                    self.handle_game_event(event).await?;
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
        &mut self,
        event: GameStateStreamEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        /*!
         * Handles the given EventType. Affects board-properties -> mut board
         */
        match event {
            GameStateStreamEvent::GameFull(full_game_data) => {
                // INIT BOARD
                // Parses first response of GameStream

                // TODO (#21): check if game is still going on

                // Check who playes white and set player-variable for the board
                self.player0_white = match Some(&full_game_data.white)
                    .expect("Unexpected value for field 'white' received in 'Board'")
                {
                    PlayedBy::User(player_info) => {
                        if player_info.id == self.user.id {
                            println!("You are playing as white");
                            true
                        } else {
                            println!("You are playing as black");
                            false
                        }
                    }
                    _ => false,
                };

                // Check if it's player0's turn
                if player0_turn(full_game_data.state.moves, self.player0_white) {
                    // TODO (#22): refactor this (to >Game< maybe?)
                    loop {
                        match self
                            .move_piece(&self.game_id, get_turn_input().await.as_str())
                            .await
                        {
                            Ok(_) => {
                                println!("Piece was moved");
                                break;
                            }
                            Err(e) => {
                                println!("Piece could not be moved {:?}", e);
                                continue;
                            }
                        }
                    }
                } else {
                    println!("It's not your turn")
                }
                // Set PlayedBy-state on board
                self.white = Some(full_game_data.white);
                self.black = Some(full_game_data.black);
            }
            GameStateStreamEvent::GameState(game_state_data) => {
                // TODO (#24): Update bit-board
                // ...
                // (1) -> player's turn now, await input: ..
                // (2) -> do nothing and await next response

                // Check if it's player0's turn

                // (1)
                if player0_turn(game_state_data.moves, self.player0_white) {
                    loop {
                        match self
                            // If so, get input, translate it to turn
                            .move_piece(&self.game_id, get_turn_input().await.as_str())
                            .await
                        {
                            Ok(_) => {
                                println!("Piece was moved");
                                break;
                            }
                            Err(e) => {
                                println!("Piece could not be moved: {:?}", e);
                                continue;
                            }
                        }
                    } // Iterate over this is long as it needs to receive a success by the API
                } else {
                    // (2)
                    println!("It's not your turn")
                }
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
