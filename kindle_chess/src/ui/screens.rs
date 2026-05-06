use std::sync::Arc;

use log::{debug, info, warn};

use crate::{
    api::oauth::{authenticate, get_user_info, load_token},
    models::{
        board_api::PlayedBy,
        chess::ChessApp,
        ui::{
            ChessAuthScreen, ChessGameScreen, Display, HomeScreen, OngoingChessGamesScreen, Screen,
            Transition,
        },
    },
    ui::{
        events::{AppEvent, RectangleExt, TouchKind},
        renderer::DrawColor,
        widgets::Button,
    },
};

// ─── HomeScreen ───────────────────────────────────────────────────────────────

impl Screen for HomeScreen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn std::error::Error>> {
        // First render: kick the silent auth bootstrap exactly once. The task
        // posts ChessReady (cached token still valid) or AuthFailed (need QR
        // flow) — both routed back through handle_event.
        if !self.auth_started {
            self.auth_started = true;
            kick_auth_bootstrap(display.event_tx.clone());
        }

        display.renderer.clear(DrawColor::White)?;
        self.chess_button.draw(&mut display.renderer)?;
        self.ongoing_games_button.draw(&mut display.renderer)?;

        display.renderer.present()?;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: AppEvent,
        display: &mut Display,
    ) -> Result<Transition, Box<dyn std::error::Error>> {
        match event {
            // Auth completed — either from our own bootstrap, or bubbled up
            // from a popped ChessAuthScreen after the QR flow finished.
            AppEvent::ChessReady(app) => {
                info!("ChessApp ready — buttons live");
                self.app = Some(app);
                Ok(Transition::Redraw)
            }

            // Bootstrap couldn't authenticate silently. Hand control to the
            // QR-flow screen; on success it'll pop and re-emit ChessReady.
            AppEvent::AuthFailed(e) => {
                warn!("Silent auth failed ({}) — pushing ChessAuthScreen", e);
                Ok(Transition::Push(Box::new(ChessAuthScreen::new())))
            }

            AppEvent::Touch(touch) => {
                if touch.kind == TouchKind::Up {
                    let Some(app) = self.app.clone() else {
                        info!("Button tap ignored — auth not yet complete");
                        return Ok(Transition::Stay);
                    };
                    if self.chess_button.rect.contains(touch.x, touch.y) {
                        info!("Chess button pressed — launching chess game");
                        return Ok(Transition::Push(Box::new(ChessGameScreen::new(app))));
                    } else if self.ongoing_games_button.rect.contains(touch.x, touch.y) {
                        info!("Ongoing-games button pressed");
                        return Ok(Transition::Push(Box::new(OngoingChessGamesScreen::new(
                            app,
                        ))));
                    }
                }

                Ok(Transition::Redraw)
            }

            AppEvent::Expose => {
                debug!("Expose event - redrawing");
                Ok(Transition::Redraw)
            }

            AppEvent::WindowUnmapped => {
                warn!("Window unmapped!");
                Ok(Transition::Stay)
            }

            AppEvent::Quit => {
                info!("Quit requested");
                Ok(Transition::Quit)
            }

            _ => Ok(Transition::Stay),
        }
    }
}

// Spawned from HomeScreen::render on first paint. Tries the cached token,
// validates it via get_user_info, and on success builds the ChessApp.
// Sends ChessReady on success, AuthFailed otherwise — never authenticate()s
// (that's ChessAuthScreen's job, since it owns the QR display).
fn kick_auth_bootstrap(tx: std::sync::mpsc::Sender<AppEvent>) {
    let maybe_token = load_token().map_err(|e| e.to_string());
    tokio::spawn(async move {
        match maybe_token {
            Ok(Some(token_info)) => {
                // Convert the non-Send Box<dyn Error> into a String before the
                // next .await so the future itself stays Send.
                let user_info = get_user_info(&token_info.access_token)
                    .await
                    .map_err(|e| e.to_string());
                match user_info {
                    Ok(user_info) => {
                        info!("Authenticated from cached token as: {}", user_info.username);
                        match ChessApp::new_online(token_info, user_info).await {
                            Ok(app) => {
                                let _ = tx.send(AppEvent::ChessReady(app));
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::AuthFailed(e.to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Cached token rejected: {} — needs re-auth", e);
                        let _ = tx.send(AppEvent::AuthFailed(e));
                    }
                }
            }
            Ok(None) => {
                info!("No cached token on disk — needs auth");
                let _ = tx.send(AppEvent::AuthFailed("no token".into()));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::AuthFailed(e));
            }
        }
    });
}

// ─── ChessGameScreen ──────────────────────────────────────────────────────────

impl Screen for ChessGameScreen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn std::error::Error>> {
        self.board.render(&mut display.renderer)?;
        self.sidebar.render(&mut display.renderer)?;
        display.renderer.present()?;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: AppEvent,
        display: &mut Display,
    ) -> Result<Transition, Box<dyn std::error::Error>> {
        match event {
            AppEvent::Touch(touch) => {
                if let Some(ev) = self.board.handle_touch(&touch) {
                    return self.handle_event(ev, display);
                }

                if let Some(ev) = self.sidebar.handle_touch(&touch) {
                    return self.handle_event(ev, display);
                }

                Ok(Transition::Redraw)
            }

            AppEvent::MoveMade(chess_move) => {
                info!(
                    "Move: {} -> {}",
                    chess_move.from.to_algebraic(),
                    chess_move.to.to_algebraic(),
                );
                // TODO: send move to backend
                Ok(Transition::Redraw)
            }

            AppEvent::SquareSelected(square) => {
                info!("Selected square: {}", square.to_algebraic());
                Ok(Transition::Redraw)
            }

            AppEvent::ShowMenu => {
                info!("Menu requested");
                // TODO: Push settings screen
                Ok(Transition::Stay)
            }

            AppEvent::ExitToMenu => {
                info!("Returning to home screen");
                Ok(Transition::Pop)
            }

            AppEvent::Expose => {
                debug!("Expose event - redrawing");
                Ok(Transition::Redraw)
            }

            AppEvent::WindowUnmapped => {
                warn!("Window unmapped!");
                Ok(Transition::Stay)
            }

            AppEvent::Quit => {
                info!("Quit requested");
                Ok(Transition::Quit)
            }

            _ => Ok(Transition::Stay),
        }
    }
}

// ─── ChessAuthScreen ──────────────────────────────────────────────────────────

impl Screen for ChessAuthScreen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn std::error::Error>> {
        // Kick the QR/PKCE flow exactly once. authenticate() will post
        // QrReady once the QR image is ready, and AuthSuccess once Lichess
        // redirects to the local callback.
        if !self.auth_started {
            self.auth_started = true;
            let tx = display.event_tx.clone();
            tokio::spawn(async move {
                match authenticate(tx.clone()).await {
                    Ok((token_info, user_info)) => {
                        info!("QR auth succeeded as: {}", user_info.username);
                        let _ = tx.send(AppEvent::AuthSuccess(token_info, user_info));
                    }
                    Err(e) => {
                        let _ = tx.send(AppEvent::AuthFailed(e.to_string()));
                    }
                }
            });
        }

        display.renderer.clear(DrawColor::White)?;
        if let Some(ref img) = self.qr_image {
            display.renderer.draw_image(
                self.qr_code.x,
                self.qr_code.y,
                self.qr_code.width,
                self.qr_code.height,
                img,
            )?;
        } else {
            display
                .renderer
                .draw_rectangle(self.qr_code, DrawColor::LightGray, true)?;
        }
        display
            .renderer
            .draw_rectangle(self.auth_status, DrawColor::LightGray, true)?;
        display.renderer.present()?;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: AppEvent,
        display: &mut Display,
    ) -> Result<Transition, Box<dyn std::error::Error>> {
        match event {
            AppEvent::AuthSuccess(token, user) => {
                // Build the ChessApp off-thread; result comes back as ChessReady.
                let tx = display.event_tx.clone();
                tokio::spawn(async move {
                    match ChessApp::new_online(token, user).await {
                        Ok(app) => {
                            let _ = tx.send(AppEvent::ChessReady(app));
                        }
                        Err(e) => {
                            let _ = tx.send(AppEvent::AuthFailed(e.to_string()));
                        }
                    }
                });
                Ok(Transition::Stay)
            }

            // Re-emit so HomeScreen (top-of-stack after our Pop) captures the
            // app, then pop ourselves off.
            AppEvent::ChessReady(app) => {
                let _ = display.event_tx.send(AppEvent::ChessReady(app));
                Ok(Transition::Pop)
            }

            AppEvent::QrReady(img) => {
                self.qr_image = Some(img);
                Ok(Transition::Redraw)
            }

            AppEvent::AuthFailed(e) => {
                warn!("QR auth failed: {}", e);
                // TODO: surface error visually + offer retry. For now stay put.
                Ok(Transition::Stay)
            }

            _ => Ok(Transition::Stay),
        }
    }
}

// ─── OngoingChessGamesScreen ──────────────────────────────────────────────────────────

impl OngoingChessGamesScreen {
    /// Spawn the ongoing-games fetch onto the tokio runtime. Idempotent w.r.t.
    /// `self.loading` — call freely from `render` (initial load) or a future
    /// reload-button handler. Result is delivered as `OngoingGamesLoaded` /
    /// `OngoingGamesFailed` over the shared event channel.
    fn kick_fetch(&mut self, display: &Display) {
        if self.loading {
            return;
        }
        let Some(api) = self.app.online_api() else {
            self.error = Some("Offline backend has no ongoing games".into());
            return;
        };
        self.loading = true;
        let tx = display.event_tx.clone();
        tokio::spawn(async move {
            match api.get_ongoing_games(8).await {
                Ok(list) => {
                    let _ = tx.send(AppEvent::OngoingGamesLoaded(Arc::new(list)));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::OngoingGamesFailed(e.to_string()));
                }
            }
        });
    }
}

impl Screen for OngoingChessGamesScreen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn std::error::Error>> {
        use crate::ui::renderer::DrawColor;

        // First paint after Push: kick off the async fetch. Subsequent renders
        // (after data arrives or on reload) skip this branch.
        if self.games.is_none() && self.error.is_none() && !self.loading {
            self.kick_fetch(display);
        }

        display.renderer.clear(DrawColor::White)?;

        let size_px = 24.0;

        if let Some(err) = &self.error {
            let label = format!("Error: {}", err);
            let (tw, th) = display.renderer.measure_text(&label, size_px);
            let tx = (1072 - tw as i16) / 2;
            let ty = (1448 - th as i16) / 2;
            display
                .renderer
                .draw_text(tx, ty, &label, size_px, DrawColor::Black)?;
        } else if let Some(games) = &self.games {
            let buttons: [&mut Button; 4] = [
                &mut self.chessgame_button_0,
                &mut self.chessgame_button_1,
                &mut self.chessgame_button_2,
                &mut self.chessgame_button_3,
            ];
            // Assumes now_playing has ≤4 entries.
            for (k, game) in games.now_playing.iter().enumerate() {
                let opp = match &game.opponent {
                    PlayedBy::User(player) => player.name.clone(),
                    PlayedBy::Ai(computer) => match computer.ai_level {
                        Some(level) => format!("AI lvl {}", level),
                        None => String::from("AI"),
                    },
                };
                let mut l = format!("VS {opp}");
                if game.is_my_turn {
                    l = format!("> {l} <");
                }
                buttons[k].label = l;
                buttons[k].draw(&mut display.renderer)?;
            }

            self.back_button.draw(&mut display.renderer)?;
            self.next_page_button.draw(&mut display.renderer)?;
            self.prev_page_button.draw(&mut display.renderer)?;
        } else {
            let label = "Loading…";
            let (tw, th) = display.renderer.measure_text(label, size_px);
            let tx = (1072 - tw as i16) / 2;
            let ty = (1448 - th as i16) / 2;
            display
                .renderer
                .draw_text(tx, ty, label, size_px, DrawColor::Black)?;
        }

        display.renderer.present()?;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: AppEvent,
        _display: &mut Display,
    ) -> Result<Transition, Box<dyn std::error::Error>> {
        match event {
            AppEvent::OngoingGamesLoaded(list) => {
                info!("Ongoing games loaded: {} entries", list.now_playing.len());
                self.games = Some(list);
                self.loading = false;
                Ok(Transition::Redraw)
            }

            AppEvent::OngoingGamesFailed(e) => {
                warn!("Ongoing games fetch failed: {}", e);
                self.error = Some(e);
                self.loading = false;
                Ok(Transition::Redraw)
            }

            AppEvent::Touch(touch) => {
                if touch.kind == TouchKind::Up && self.back_button.rect.contains(touch.x, touch.y) {
                    return Ok(Transition::Pop);
                }
                Ok(Transition::Stay)
            }

            AppEvent::Expose => Ok(Transition::Redraw),

            AppEvent::WindowUnmapped => {
                warn!("Window unmapped!");
                Ok(Transition::Stay)
            }

            AppEvent::Quit => Ok(Transition::Quit),

            _ => Ok(Transition::Stay),
        }
    }
}
