use std::sync::Arc;

use log::{debug, info, warn};

use crate::{
    api::oauth::{authenticate, get_user_info, load_token},
    models::{
        chess::ChessApp,
        ui::{
            ChessAuthScreen, ChessGameScreen, Display, HomeScreen, OngoingChessGamesScreen, Screen,
            Transition,
        },
    },
    ui::events::{AppEvent, RectangleExt, TouchKind},
};

// ─── HomeScreen ───────────────────────────────────────────────────────────────

impl Screen for HomeScreen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn std::error::Error>> {
        use crate::ui::renderer::DrawColor;
        display.renderer.clear(DrawColor::White)?;
        display
            .renderer
            .draw_rectangle(self.chess_button, DrawColor::White, true)?;
        display
            .renderer
            .draw_rectangle(self.chess_button, DrawColor::Black, false)?;

        let label = "CHESS";
        let size_px = 64.0;
        let (tw, th) = display.renderer.measure_text(label, size_px);
        let tx = self.chess_button.x + (self.chess_button.width as i16 - tw as i16) / 2;
        let ty = self.chess_button.y + (self.chess_button.height as i16 - th as i16) / 2;
        display
            .renderer
            .draw_text(tx, ty, label, size_px, DrawColor::Black)?;

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
                if touch.kind == TouchKind::Up {
                    if self.chess_button.contains(touch.x, touch.y) {
                        info!("Chess button pressed — launching chess");
                        info!("Checking Authentication Status");
                        // TODO: Also check for internet connection before sending to auth screen
                        //
                        //
                        let maybe_token = load_token().map_err(|e| e.to_string());
                        let tx = display.event_tx.clone(); // clone sender for auth task

                        // Spawn async work on tokio runtime
                        tokio::spawn(async move {
                            match maybe_token {
                                // Token exists on disk
                                Ok(Some(token_info)) => {
                                    match get_user_info(&token_info.access_token)
                                        .await
                                        .map_err(|e| e.to_string())
                                    {
                                        Ok(user_info) => {
                                            info!(
                                                "Successfully authenticated from token as: {}",
                                                user_info.username
                                            );
                                            let _ = tx
                                                .send(AppEvent::AuthSuccess(token_info, user_info));
                                        }
                                        Err(_) => {
                                            // Stale token, reauthenticate
                                            // Send EventSender so QrReady-Event can be sent
                                            match authenticate(tx.clone())
                                                .await
                                                .map_err(|e| e.to_string())
                                            {
                                                Ok((token_info, user_info)) => {
                                                    let _ = tx.send(AppEvent::AuthSuccess(
                                                        token_info, user_info,
                                                    ));
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(AppEvent::AuthFailed(e));
                                                }
                                            }
                                        }
                                    }
                                }
                                // (Re-)Authentication needed
                                Ok(None) => {
                                    // Stale token, reauthenticate
                                    // Send EventSender so QrReady-Event can be sent
                                    match authenticate(tx.clone()).await.map_err(|e| e.to_string())
                                    {
                                        Ok((token_info, user_info)) => {
                                            info!(
                                                "Successfully re-authenticated as: {}",
                                                user_info.username
                                            );
                                            let _ = tx
                                                .send(AppEvent::AuthSuccess(token_info, user_info));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(AppEvent::AuthFailed(e));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(AppEvent::AuthFailed(e));
                                }
                            }
                        });

                        // TODO: push ChessAuthScreen or ChessGameScreen depending on auth state
                        return Ok(Transition::Push(Box::new(ChessAuthScreen::new())));
                    }
                    // TODO: else if self.other_button.contains(touch.x, touch.x) {...}
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
        display
            .renderer
            .clear(crate::ui::renderer::DrawColor::White)?;
        if let Some(ref img) = self.qr_image {
            display.renderer.draw_image(
                self.qr_code.x,
                self.qr_code.y,
                self.qr_code.width,
                self.qr_code.height,
                img,
            )?;
        } else {
            display.renderer.draw_rectangle(
                self.qr_code,
                crate::ui::renderer::DrawColor::LightGray,
                true,
            )?;
        }
        display.renderer.draw_rectangle(
            self.auth_status,
            crate::ui::renderer::DrawColor::LightGray,
            true,
        )?;
        display.renderer.present()?;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: AppEvent,
        display: &mut Display,
    ) -> Result<Transition, Box<dyn std::error::Error>> {
        match event {
            AppEvent::Touch(touch) => Ok(Transition::Stay),

            AppEvent::AuthSuccess(token, user) => {
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
                Ok(Transition::Stay) // stays on auth screen until ChessReady arrives
            }

            AppEvent::ChessReady(app) => Ok(Transition::Push(Box::new(ChessGameScreen::new(app)))),
            AppEvent::QrReady(img) => {
                self.qr_image = Some(img);
                Ok(Transition::Redraw)
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

        let size_px = 64.0;

        if let Some(err) = &self.error {
            let label = format!("Error: {}", err);
            let (tw, th) = display.renderer.measure_text(&label, size_px);
            let tx = (1072 - tw as i16) / 2;
            let ty = (1448 - th as i16) / 2;
            display
                .renderer
                .draw_text(tx, ty, &label, size_px, DrawColor::Black)?;
        } else if let Some(games) = &self.games {
            let buttons = [
                self.chessgame_button_0,
                self.chessgame_button_1,
                self.chessgame_button_2,
                self.chessgame_button_3,
            ];
            for (i, btn) in buttons.iter().enumerate() {
                display.renderer.draw_rectangle(*btn, DrawColor::White, true)?;
                display.renderer.draw_rectangle(*btn, DrawColor::Black, false)?;

                let label = match games.now_playing.get(i) {
                    Some(g) => format!("{}  vs  opp", g.game_id),
                    None => "—".to_string(),
                };
                let (tw, th) = display.renderer.measure_text(&label, size_px);
                let tx = btn.x + (btn.width as i16 - tw as i16) / 2;
                let ty = btn.y + (btn.height as i16 - th as i16) / 2;
                display
                    .renderer
                    .draw_text(tx, ty, &label, size_px, DrawColor::Black)?;
            }
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
                if touch.kind == TouchKind::Up && self.back_button.contains(touch.x, touch.y) {
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
