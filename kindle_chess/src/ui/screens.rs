use log::{debug, info, warn};

use crate::{
    api::oauth::{authenticate, get_user_info, load_token},
    models::{
        chess::ChessApp,
        ui::{ChessAuthScreen, ChessGameScreen, Display, HomeScreen, Screen, Transition},
    },
    ui::events::{AppEvent, RectangleExt, TouchKind},
};

// ─── HomeScreen ───────────────────────────────────────────────────────────────

impl Screen for HomeScreen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn std::error::Error>> {
        display
            .renderer
            .clear(crate::ui::renderer::DrawColor::White)?;
        display.renderer.draw_rectangle(
            self.chess_button,
            crate::ui::renderer::DrawColor::LightGray,
            true,
        )?;
        display.renderer.draw_rectangle(
            self.chess_button,
            crate::ui::renderer::DrawColor::Black,
            false,
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
                                            match authenticate().await.map_err(|e| e.to_string()) {
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
                                    match authenticate().await.map_err(|e| e.to_string()) {
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
        display.renderer.draw_rectangle(
            self.qr_code,
            crate::ui::renderer::DrawColor::LightGray,
            true,
        )?;
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

            _ => Ok(Transition::Stay),
        }
    }
}
