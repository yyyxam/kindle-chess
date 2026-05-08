use std::sync::Arc;

use log::{debug, info, warn};

use crate::{
    api::{
        github::{check_for_update, UpdateInfo},
        oauth::{authenticate, get_user_info, load_token},
        update::apply_update,
    },
    models::{
        board_api::PlayedBy,
        chess::ChessApp,
        ui::{
            ChessAuthScreen, ChessGameScreen, Display, HomeScreen, OngoingChessGamesScreen, Screen,
            SettingsScreen, Transition, UpdateScreen, UpdateState,
        },
    },
    ui::{
        events::{AppEvent, RectangleExt, TouchKind},
        renderer::DrawColor,
        widgets::Button,
    },
    version,
};
use std::sync::mpsc::Sender;

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
        self.settings_button.draw(&mut display.renderer)?;

        display.renderer.present()?;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: AppEvent,
        _display: &mut Display,
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
                    // Settings is reachable regardless of auth state — an
                    // offline user still needs the update entry point.
                    if self.settings_button.rect.contains(touch.x, touch.y) {
                        info!("Settings button pressed");
                        return Ok(Transition::Push(Box::new(SettingsScreen::new())));
                    }

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
                        let _ = tx.send(AppEvent::ChessReady(ChessApp::new_online(
                            token_info, user_info,
                        )));
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
        // First paint after Push: spawn the game-state stream task. It owns a
        // clone of BoardAPI<InGame>; everything we care about comes back as
        // GameFullReceived / TurnChanged events (see kick_game_stream).
        if !self.stream_started {
            self.stream_started = true;
            kick_game_stream(&self.app, display.event_tx.clone());
        }

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
            AppEvent::GameFullReceived {
                white,
                black,
                player0_white,
                turn,
                board,
                last_move,
            } => {
                self.app
                    .apply_game_full(white, black, player0_white, turn.clone());
                // Orient the board so the local player's pieces are on the
                // bottom rank — flip when player0 is black. set_flipped is a
                // no-op when orientation is unchanged, so re-applying on
                // every GameFull is cheap.
                self.board.set_flipped(!player0_white);
                self.board.set_position(board);
                self.board.set_last_move(last_move);
                self.sidebar.set_turn(turn);
                Ok(Transition::Redraw)
            }

            AppEvent::TurnChanged {
                turn,
                board,
                last_move,
            } => {
                self.app.apply_turn(turn.clone());
                self.board.set_position(board);
                self.board.set_last_move(last_move);
                self.sidebar.set_turn(turn);
                Ok(Transition::Redraw)
            }

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
                let Some(uci) = self.board.move_to_uci(chess_move) else {
                    warn!("MoveMade before board position loaded — dropping move");
                    return Ok(Transition::Redraw);
                };
                if let Some(api) = self.app.online_in_game_api() {
                    tokio::spawn(async move {
                        if let Err(e) = api.move_piece(&uci).await {
                            warn!("move_piece({}) failed: {}", uci, e);
                        }
                    });
                } else {
                    warn!("MoveMade with no online in-game backend — ignored");
                }
                Ok(Transition::Redraw)
            }

            AppEvent::SquareSelected(square) => {
                info!("Selected square: {}", square.to_algebraic());
                Ok(Transition::Redraw)
            }

            AppEvent::ShowMenu => {
                info!("Menu requested — returning to home screen");
                Ok(Transition::Pop)
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

// Spawns the game-state stream onto the tokio runtime. The task owns a fresh
// clone of `BoardAPI<InGame>`; mutations to the clone's `state` are local
// bookkeeping. Every state change the screen needs is sent back as an
// `AppEvent`. No-op when the screen wasn't pushed with an in-game backend
// (e.g. the Demo button path, which still uses an Idle ChessApp).
fn kick_game_stream(app: &ChessApp, tx: Sender<AppEvent>) {
    let Some(mut api) = app.online_in_game_api() else {
        warn!("ChessGameScreen has no in-game backend — skipping stream");
        return;
    };
    tokio::spawn(async move {
        if let Err(e) = api.stream_game_event(tx).await {
            warn!("Game-state stream errored: {}", e);
        }
    });
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
                // ChessApp::new_online is sync (no I/O at construction), so just
                // post ChessReady directly.
                let _ = display
                    .event_tx
                    .send(AppEvent::ChessReady(ChessApp::new_online(token, user)));
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

const GAMES_PER_PAGE: usize = 4;

impl OngoingChessGamesScreen {
    /// Spawn the ongoing-games fetch onto the tokio runtime. Idempotent w.r.t.
    /// `self.loading` — call freely from `render` (initial load) or a future
    /// reload-button handler. Result is delivered as `OngoingGamesLoaded` /
    /// `OngoingGamesFailed` over the shared event channel.
    fn kick_fetch(&mut self, display: &Display) {
        if self.loading {
            return;
        }
        let Some(api) = self.app.online_idle_api() else {
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

    fn page_count(&self) -> usize {
        match &self.games {
            Some(g) => g.now_playing.len().div_ceil(GAMES_PER_PAGE),
            None => 0,
        }
    }

    /// Bake labels for `index`'s page of games into the four chess-game buttons,
    /// and update `self.page_index`. No-op when `games` hasn't loaded yet — the
    /// `OngoingGamesLoaded` handler calls this again once data arrives. Slots
    /// past the available game count get a "-" so they're visibly inert; the
    /// touch handler also short-circuits taps on those slots.
    fn set_page(&mut self, index: usize) {
        self.page_index = index;
        let Some(games) = self.games.as_ref() else {
            return;
        };
        let buttons: [&mut Button; 4] = [
            &mut self.chessgame_button_0,
            &mut self.chessgame_button_1,
            &mut self.chessgame_button_2,
            &mut self.chessgame_button_3,
        ];
        let start = index * GAMES_PER_PAGE;
        for (k, btn) in buttons.into_iter().enumerate() {
            match games.now_playing.get(start + k) {
                Some(game) => {
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
                    btn.label = l;
                }
                None => {
                    btn.label = "-".to_string();
                }
            }
        }
    }

    /// Resolve which game `chessgame_button_{k}` currently maps to, given the
    /// current page. Returns `None` for slots that are inert ("-"-labelled
    /// because the current page has fewer than 4 games left).
    fn game_at_slot(&self, k: usize) -> Option<&crate::models::board_api::GameData> {
        self.games
            .as_ref()?
            .now_playing
            .get(self.page_index * GAMES_PER_PAGE + k)
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
        } else if self.games.is_some() {
            // Labels were baked into the buttons by `set_page` (called from
            // OngoingGamesLoaded and from next/prev taps), so render is just
            // a draw pass.
            self.chessgame_button_0.draw(&mut display.renderer)?;
            self.chessgame_button_1.draw(&mut display.renderer)?;
            self.chessgame_button_2.draw(&mut display.renderer)?;
            self.chessgame_button_3.draw(&mut display.renderer)?;
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
                // Bake labels for the current page now that we have data.
                self.set_page(self.page_index);
                Ok(Transition::Redraw)
            }

            AppEvent::OngoingGamesFailed(e) => {
                warn!("Ongoing games fetch failed: {}", e);
                self.error = Some(e);
                self.loading = false;
                Ok(Transition::Redraw)
            }

            AppEvent::Touch(touch) => {
                if touch.kind != TouchKind::Up {
                    return Ok(Transition::Stay);
                }

                if self.back_button.rect.contains(touch.x, touch.y) {
                    return Ok(Transition::Pop);
                }

                if self.next_page_button.rect.contains(touch.x, touch.y) {
                    let last = self.page_count().saturating_sub(1);
                    if self.page_index < last {
                        self.set_page(self.page_index + 1);
                        return Ok(Transition::Redraw);
                    }
                    return Ok(Transition::Stay);
                }

                if self.prev_page_button.rect.contains(touch.x, touch.y) {
                    if self.page_index > 0 {
                        self.set_page(self.page_index - 1);
                        return Ok(Transition::Redraw);
                    }
                    return Ok(Transition::Stay);
                }

                // Game-button taps. Resolve via `game_at_slot` so paginated
                // slots without backing data are inert.
                let game_buttons = [
                    &self.chessgame_button_0,
                    &self.chessgame_button_1,
                    &self.chessgame_button_2,
                    &self.chessgame_button_3,
                ];
                for (k, btn) in game_buttons.into_iter().enumerate() {
                    if !btn.rect.contains(touch.x, touch.y) {
                        continue;
                    }
                    let Some(game) = self.game_at_slot(k) else {
                        return Ok(Transition::Stay);
                    };
                    info!(
                        "Picked ongoing game (page {}, slot {}): {}",
                        self.page_index, k, game.game_id
                    );
                    let game_app = self
                        .app
                        .clone()
                        .attach_game(game.game_id.clone(), game.is_my_turn);
                    return Ok(Transition::Push(Box::new(ChessGameScreen::new(game_app))));
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

// ─── SettingsScreen ───────────────────────────────────────────────────────────

impl Screen for SettingsScreen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn std::error::Error>> {
        display.renderer.clear(DrawColor::White)?;

        // Title
        let title = "Settings";
        let title_size = 56.0;
        let (tw, _) = display.renderer.measure_text(title, title_size);
        display
            .renderer
            .draw_text((1072 - tw as i16) / 2, 120, title, title_size, DrawColor::Black)?;

        // Version block
        let info_size = 28.0;
        let lines = [
            format!("Version:  {}", version::VERSION),
            format!("Commit:   {}", version::GIT_SHA),
            format!("Built:    {}", version::BUILD_TIMESTAMP),
        ];
        let mut y: i16 = 240;
        for line in &lines {
            display
                .renderer
                .draw_text(120, y, line, info_size, DrawColor::Black)?;
            y += 40;
        }

        self.check_update_button.draw(&mut display.renderer)?;
        self.back_button.draw(&mut display.renderer)?;
        display.renderer.present()?;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: AppEvent,
        _display: &mut Display,
    ) -> Result<Transition, Box<dyn std::error::Error>> {
        match event {
            AppEvent::Touch(touch) => {
                if touch.kind != TouchKind::Up {
                    return Ok(Transition::Stay);
                }
                if self.check_update_button.rect.contains(touch.x, touch.y) {
                    info!("Check-for-updates pressed");
                    return Ok(Transition::Push(Box::new(UpdateScreen::new())));
                }
                if self.back_button.rect.contains(touch.x, touch.y) {
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

// ─── UpdateScreen ─────────────────────────────────────────────────────────────

impl Screen for UpdateScreen {
    fn render(&mut self, display: &mut Display) -> Result<(), Box<dyn std::error::Error>> {
        // Auto-kick the check on first paint.
        if !self.check_started {
            self.check_started = true;
            kick_update_check(display.event_tx.clone());
        }

        display.renderer.clear(DrawColor::White)?;

        // Title
        let title = "Update";
        let title_size = 56.0;
        let (tw, _) = display.renderer.measure_text(title, title_size);
        display
            .renderer
            .draw_text((1072 - tw as i16) / 2, 120, title, title_size, DrawColor::Black)?;

        // State-driven body. The action button is only drawn (and only live in
        // the touch handler) when there is something meaningful to do.
        let (lines, action_label, action_visible) = match &self.state {
            UpdateState::Checking => (
                vec!["Checking for updates…".to_string()],
                "Apply",
                false,
            ),
            UpdateState::UpToDate => (
                vec![format!("You're up to date — v{}", version::VERSION)],
                "Apply",
                false,
            ),
            UpdateState::Available(info) => (
                vec![
                    format!("Update available: v{} → v{}", info.current, info.latest),
                    format!("({} bytes)", info.asset_size),
                    String::new(),
                    info.release_name.clone(),
                ],
                "Apply",
                true,
            ),
            UpdateState::Downloading => (
                vec!["Downloading and verifying…".to_string()],
                "Apply",
                false,
            ),
            UpdateState::Applied => (
                vec![
                    "Update applied.".to_string(),
                    "Quit and relaunch from the KUAL menu.".to_string(),
                ],
                "Quit",
                true,
            ),
            UpdateState::Failed(err) => (
                vec!["Update failed:".to_string(), err.clone()],
                "Apply",
                false,
            ),
        };

        let info_size = 28.0;
        let mut y: i16 = 280;
        for line in &lines {
            let (lw, _) = display.renderer.measure_text(line, info_size);
            let lx = (1072 - lw as i16) / 2;
            display
                .renderer
                .draw_text(lx, y, line, info_size, DrawColor::Black)?;
            y += 40;
        }

        if action_visible {
            self.action_button.label = action_label.to_string();
            self.action_button.draw(&mut display.renderer)?;
        }
        self.back_button.draw(&mut display.renderer)?;

        display.renderer.present()?;
        Ok(())
    }

    fn handle_event(
        &mut self,
        event: AppEvent,
        display: &mut Display,
    ) -> Result<Transition, Box<dyn std::error::Error>> {
        match event {
            AppEvent::UpdateAvailable(info) => {
                info!("Update available: v{} → v{}", info.current, info.latest);
                self.state = UpdateState::Available(info);
                Ok(Transition::Redraw)
            }
            AppEvent::UpdateUpToDate => {
                self.state = UpdateState::UpToDate;
                Ok(Transition::Redraw)
            }
            AppEvent::UpdateCheckFailed(e) => {
                warn!("Update check failed: {}", e);
                self.state = UpdateState::Failed(format!("Check failed: {}", e));
                Ok(Transition::Redraw)
            }
            AppEvent::UpdateApplied => {
                info!("Update applied");
                self.state = UpdateState::Applied;
                Ok(Transition::Redraw)
            }
            AppEvent::UpdateApplyFailed(e) => {
                warn!("Update apply failed: {}", e);
                self.state = UpdateState::Failed(format!("Apply failed: {}", e));
                Ok(Transition::Redraw)
            }

            AppEvent::Touch(touch) => {
                if touch.kind != TouchKind::Up {
                    return Ok(Transition::Stay);
                }
                if self.back_button.rect.contains(touch.x, touch.y) {
                    return Ok(Transition::Pop);
                }
                if self.action_button.rect.contains(touch.x, touch.y) {
                    match &self.state {
                        UpdateState::Available(info) => {
                            let info = info.clone();
                            self.state = UpdateState::Downloading;
                            kick_update_apply(info, display.event_tx.clone());
                            return Ok(Transition::Redraw);
                        }
                        UpdateState::Applied => {
                            // KUAL will relaunch the binary the next time the
                            // user taps the menu entry — and exec(2) picks up
                            // the new file at the path we just renamed onto.
                            return Ok(Transition::Quit);
                        }
                        _ => {}
                    }
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

fn kick_update_check(tx: Sender<AppEvent>) {
    tokio::spawn(async move {
        match check_for_update().await {
            Ok(Some(info)) => {
                let _ = tx.send(AppEvent::UpdateAvailable(info));
            }
            Ok(None) => {
                let _ = tx.send(AppEvent::UpdateUpToDate);
            }
            Err(e) => {
                let _ = tx.send(AppEvent::UpdateCheckFailed(e.to_string()));
            }
        }
    });
}

fn kick_update_apply(info: UpdateInfo, tx: Sender<AppEvent>) {
    tokio::spawn(async move {
        match apply_update(&info).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::UpdateApplied);
            }
            Err(e) => {
                let _ = tx.send(AppEvent::UpdateApplyFailed(e.to_string()));
            }
        }
    });
}
