use crate::api::oauth::authenticated_request;
use crate::app::game::player0_turn;
use crate::models::board_api::{
    BoardAPI, GameDataList, GameStateStreamEvent, Idle, InGame, PlayedBy, StreamEvent, Turn,
};
use crate::models::oauth::{HttpMethod, LichessUser, TokenInfo};
use crate::ui::events::AppEvent;
use futures::StreamExt;
use log::{info, warn};
use reqwest_streams::JsonStreamResponse;
use std::sync::mpsc::Sender;

// State-agnostic operations: ongoing-games list and the account event stream
// are valid in both Idle and InGame.
impl<S> BoardAPI<S> {
    pub async fn get_ongoing_games(
        &self,
        n: u8,
    ) -> Result<GameDataList, Box<dyn std::error::Error>> {
        let url = format!("{}/account/playing?nb={}", env!("LICHESS_API_BASE"), n);
        let response = authenticated_request(url, &self.token, HttpMethod::GET).await?;

        if !&response.status().is_success() {
            return Err(format!("Failed to retrieve ongoing games: {}", &response.status()).into());
        }

        let bytes = response.bytes().await?;

        let data: GameDataList = serde_json::from_slice(&bytes).map_err(|e| {
            warn!(
                "Failed to parse JSON. Raw response: {}",
                String::from_utf8_lossy(&bytes)
            );
            warn!("Parse error: {}", e);
            e
        })?;
        info!("The received and parsed data {:?}", data);
        Ok(data)
    }

    pub async fn stream_event(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/stream/event", env!("LICHESS_API_BASE"));

        info!("Getting event stream");
        let mut response = authenticated_request(url, &self.token, HttpMethod::GET)
            .await?
            .json_nl_stream::<StreamEvent>(1024);

        while let Some(result) = response.next().await {
            match result {
                Ok(event) => {
                    info!("Received event: {:?}", event);
                    self.handle_event(event).await?;
                }
                Err(e) => {
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
            StreamEvent::GameStart(_) => info!("Issa GameStartEvent"),
            StreamEvent::GameFinish(_) => info!("Issa GameFinishEvent"),
            StreamEvent::Challenge(_) => info!("Issa ChallengeEvent"),
            StreamEvent::ChallengeDeclined(_) => info!("Issa ChallengeDeclinedEvent"),
        }
        Ok(())
    }
}

impl BoardAPI<Idle> {
    pub fn new(token: TokenInfo, user: LichessUser) -> Self {
        Self {
            token,
            user,
            state: Idle,
        }
    }

    /// Consume the idle API and produce an in-game one scoped to `game_id`.
    /// `my_turn` is the snapshot from `GameData.is_my_turn` at attach time so
    /// the sidebar can render something before the game-state stream catches up.
    pub fn attach_game(self, game_id: String, my_turn: bool) -> BoardAPI<InGame> {
        BoardAPI {
            token: self.token,
            user: self.user,
            state: InGame {
                game_id,
                white: None,
                black: None,
                player0_white: false,
                turn: if my_turn { Turn::Playing } else { Turn::Waiting },
            },
        }
    }
}

impl BoardAPI<InGame> {
    pub fn game_id(&self) -> &str {
        &self.state.game_id
    }

    pub fn turn(&self) -> &Turn {
        &self.state.turn
    }

    pub async fn move_piece(&self, board_move: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/board/game/{}/move/{}",
            env!("LICHESS_API_BASE"),
            self.state.game_id,
            board_move
        );
        info!("Sending move {} for game {}", board_move, self.state.game_id);

        let response = authenticated_request(url, &self.token, HttpMethod::POST).await?;

        if !response.status().is_success() {
            return Err(format!("Failed to move piece: {}", response.status()).into());
        }
        info!("Piece moved successfully");
        Ok(())
    }

    pub async fn resign_game(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/board/game/{}/resign",
            env!("LICHESS_API_BASE"),
            self.state.game_id
        );

        let response = authenticated_request(url, &self.token, HttpMethod::POST).await?;

        if !response.status().is_success() {
            return Err(format!("Failed to resign game: {}", response.status()).into());
        }
        info!("Game resigned");
        Ok(())
    }

    pub async fn abort_game(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/board/game/{}/abort",
            env!("LICHESS_API_BASE"),
            self.state.game_id
        );

        let response = authenticated_request(url, &self.token, HttpMethod::POST).await?;

        if !response.status().is_success() {
            return Err(format!("Failed to abort game: {}", response.status()).into());
        }
        info!("Game aborted");
        Ok(())
    }

    /// Open the game-state stream and drive it to completion.
    ///
    /// This runs inside a tokio task that owns its own clone of the API, so
    /// the local `&mut self` mutations to `self.state` are bookkeeping for
    /// computing whose turn it is on subsequent `GameState` events — they do
    /// **not** propagate back to the screen. Every state change the screen
    /// cares about is shipped as an `AppEvent` through `tx`. The screen
    /// applies those events to *its own* `ChessApp` copy.
    pub async fn stream_game_event(
        &mut self,
        tx: Sender<AppEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/board/game/stream/{}",
            env!("LICHESS_API_BASE"),
            self.state.game_id
        );
        info!("Game-state stream started for {}", self.state.game_id);

        let mut response = authenticated_request(url, &self.token, HttpMethod::STREAM)
            .await?
            .json_nl_stream::<GameStateStreamEvent>(1024);

        while let Some(result) = response.next().await {
            match result {
                Ok(event) => {
                    info!("Received event: {:?}", event);
                    if let Err(e) = self.handle_game_event(event, &tx).await {
                        warn!("Error while handling game event: {e}");
                    }
                }
                Err(e) => {
                    // Ignore the stream ping (= empty line)
                    if e.to_string().contains("EOF while parsing") {
                        continue;
                    }
                    warn!("Error parsing event: {}", e);
                }
            }
        }
        Ok(())
    }

    async fn handle_game_event(
        &mut self,
        event: GameStateStreamEvent,
        tx: &Sender<AppEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            GameStateStreamEvent::GameFull(full) => {
                let player0_white = matches!(&full.white, PlayedBy::User(p) if p.id == self.user.id);
                let my_turn = player0_turn(full.state.moves.clone(), player0_white);
                let turn = if my_turn { Turn::Playing } else { Turn::Waiting };

                // Update local bookkeeping so subsequent GameState events can
                // resolve whose-turn-it-is from `player0_white`.
                self.state.white = Some(full.white.clone());
                self.state.black = Some(full.black.clone());
                self.state.player0_white = player0_white;
                self.state.turn = turn.clone();

                let _ = tx.send(AppEvent::GameFullReceived {
                    white: full.white,
                    black: full.black,
                    player0_white,
                    turn,
                });
            }
            GameStateStreamEvent::GameState(state) => {
                let my_turn = player0_turn(state.moves, self.state.player0_white);
                let turn = if my_turn { Turn::Playing } else { Turn::Waiting };
                self.state.turn = turn.clone();
                let _ = tx.send(AppEvent::TurnChanged(turn));
            }
            GameStateStreamEvent::GameOver(over) => {
                info!("Game is over. Winner is {}", over.winner);
                let turn = Turn::Over {
                    winner: Some(over.winner),
                };
                self.state.turn = turn.clone();
                let _ = tx.send(AppEvent::TurnChanged(turn));
            }
            GameStateStreamEvent::ChatLine(_) => info!("Issa ChatlineEvent"),
            GameStateStreamEvent::OpponentGone(_) => info!("Issa OpponentGoneEvent"),
        }
        Ok(())
    }
}
