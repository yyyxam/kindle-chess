use log::{debug, error, info, warn};
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::wrapper::ConnectionExt;
use x11rb::COPY_DEPTH_FROM_PARENT;

const KINDLE_WIDTH: u16 = 1072;
const KINDLE_HEIGHT: u16 = 1448;
const SQUARE_SIZE: u16 = 134; // 1072 / 8 = 134
const EXIT_ZONE_HEIGHT: u16 = 100; // Bottom 100 pixels for exit

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp(None)
        .init();

    info!("Starting Kindle X11 Chess Test");
    info!("EXIT: Touch the bottom 100 pixels or triple-tap anywhere");

    // Connect to X11
    let (conn, screen_num) = match x11rb::connect(Some(":0.0")) {
        Ok(result) => result,
        Err(_) => {
            warn!("Failed to connect to :0.0, trying :0");
            x11rb::connect(Some(":0"))?
        }
    };

    info!("Connected to X11, screen number: {}", screen_num);

    let screen = &conn.setup().roots[screen_num];
    info!(
        "Screen: {}x{}, depth: {}",
        screen.width_in_pixels, screen.height_in_pixels, screen.root_depth
    );

    // Create our window
    let win = conn.generate_id()?;

    let win_aux = CreateWindowAux::new()
        .background_pixel(screen.white_pixel)
        .override_redirect(1)
        .event_mask(
            EventMask::EXPOSURE
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::KEY_PRESS
                | EventMask::STRUCTURE_NOTIFY,
        );

    conn.create_window(
        COPY_DEPTH_FROM_PARENT,
        win,
        screen.root,
        0,
        0,
        KINDLE_WIDTH,
        KINDLE_HEIGHT,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &win_aux,
    )?;

    info!("Created window: 0x{:x}", win);

    // Set window name
    conn.change_property8(
        PropMode::REPLACE,
        win,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        b"ChessTest - Touch bottom to exit",
    )?;

    // Create GCs for drawing
    let black_gc = create_gc(&conn, win, 0)?;
    let white_gc = create_gc(&conn, win, 255)?;
    let gray_gc = create_gc(&conn, win, 128)?;
    let red_gc = create_gc(&conn, win, 64)?; // Will appear as dark gray

    // Map window
    conn.map_window(win)?;
    conn.configure_window(win, &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE))?;
    conn.flush()?;

    info!("Window mapped and raised");

    // Draw initial board
    draw_chessboard(&conn, win, black_gc, white_gc)?;
    draw_exit_zone(&conn, win, red_gc)?;
    conn.flush()?;

    // Event handling variables
    let mut touch_x: i16 = -1;
    let mut touch_y: i16 = -1;
    let mut event_count = 0;

    // Triple-tap detection
    let mut tap_times: Vec<Instant> = Vec::new();
    let mut last_tap_pos: Option<(i16, i16)> = None;
    const TRIPLE_TAP_TIME: Duration = Duration::from_millis(500);
    const TRIPLE_TAP_RADIUS: i16 = 50;

    // Timeout exit (optional - set to None to disable)
    let timeout_duration = Some(Duration::from_secs(300)); // 5 minutes
    let start_time = Instant::now();

    info!("Entering event loop...");
    info!("Exit methods:");
    info!("  1. Touch the red zone at bottom");
    info!("  2. Triple-tap anywhere");
    info!("  3. Press any hardware button (if detected)");
    if let Some(timeout) = timeout_duration {
        info!("  4. Auto-exit after {} seconds", timeout.as_secs());
    }

    // Main event loop
    loop {
        // Check timeout
        if let Some(timeout) = timeout_duration {
            if start_time.elapsed() > timeout {
                info!("Timeout reached, exiting...");
                break;
            }
        }

        // Poll for events with timeout
        let event = match conn.poll_for_event()? {
            Some(event) => event,
            None => {
                // No event, sleep a bit and continue
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
        };

        event_count += 1;

        match event {
            Event::Expose(e) => {
                info!(
                    "Expose event #{}: {}x{} at ({},{})",
                    event_count, e.width, e.height, e.x, e.y
                );

                // Redraw everything
                draw_chessboard(&conn, win, black_gc, white_gc)?;
                draw_exit_zone(&conn, win, red_gc)?;

                if touch_x >= 0 && touch_y >= 0 {
                    draw_touch_indicator(&conn, win, gray_gc, touch_x, touch_y)?;
                }

                draw_text_info(&conn, win, black_gc, event_count)?;
                conn.flush()?;
            }

            Event::ButtonPress(e) => {
                info!("Touch DOWN at ({}, {})", e.event_x, e.event_y);

                // Check if touch is in exit zone
                if e.event_y >= (KINDLE_HEIGHT - EXIT_ZONE_HEIGHT) as i16 {
                    info!("Touch in EXIT ZONE - exiting application!");
                    break;
                }

                // Triple-tap detection
                let now = Instant::now();
                let tap_pos = (e.event_x, e.event_y);

                // Check if this tap is close to previous taps
                if let Some(last_pos) = last_tap_pos {
                    let dx = (tap_pos.0 - last_pos.0).abs();
                    let dy = (tap_pos.1 - last_pos.1).abs();

                    if dx <= TRIPLE_TAP_RADIUS && dy <= TRIPLE_TAP_RADIUS {
                        // Close to last position
                        tap_times.push(now);

                        // Remove old taps
                        tap_times.retain(|t| now.duration_since(*t) < TRIPLE_TAP_TIME);

                        if tap_times.len() >= 3 {
                            info!("TRIPLE TAP DETECTED - exiting!");
                            break;
                        }
                    } else {
                        // Too far, reset
                        tap_times.clear();
                        tap_times.push(now);
                        last_tap_pos = Some(tap_pos);
                    }
                } else {
                    // First tap
                    tap_times.clear();
                    tap_times.push(now);
                    last_tap_pos = Some(tap_pos);
                }

                touch_x = e.event_x;
                touch_y = e.event_y;

                // Calculate chess square if in chess area
                if e.event_y < (8 * SQUARE_SIZE) as i16 {
                    let square_x = (e.event_x / SQUARE_SIZE as i16) as usize;
                    let square_y = (e.event_y / SQUARE_SIZE as i16) as usize;
                    let file = ('a' as u8 + square_x as u8) as char;
                    let rank = 8 - square_y;
                    info!("Chess square: {}{}", file, rank);
                }

                // Visual feedback
                draw_touch_indicator(&conn, win, gray_gc, touch_x, touch_y)?;
                conn.flush()?;
            }

            Event::ButtonRelease(e) => {
                info!("Touch UP at ({}, {})", e.event_x, e.event_y);

                // Clear touch indicator
                draw_chessboard(&conn, win, black_gc, white_gc)?;
                draw_exit_zone(&conn, win, red_gc)?;
                draw_text_info(&conn, win, black_gc, event_count)?;
                touch_x = -1;
                touch_y = -1;

                conn.flush()?;
            }

            Event::KeyPress(e) => {
                info!("Key press: keycode={} - EXITING on any key!", e.detail);
                // Exit on any hardware button press
                break;
            }

            Event::UnmapNotify(e) => {
                warn!("Window unmapped! Event window: 0x{:x}", e.event);
                conn.map_window(win)?;
                conn.flush()?;
            }

            Event::ConfigureNotify(e) => {
                debug!(
                    "Configure notify: {}x{} at ({},{})",
                    e.width, e.height, e.x, e.y
                );
            }

            Event::Error(e) => {
                error!("X11 Error: {:?}", e);
            }

            _ => {
                debug!("Event #{}: {:?}", event_count, event);
            }
        }
    }

    // Cleanup
    info!("Cleaning up and exiting...");
    conn.destroy_window(win)?;
    conn.flush()?;

    info!("Application terminated successfully");
    Ok(())
}

fn create_gc(
    conn: &impl Connection,
    window: Window,
    pixel_value: u32,
) -> Result<Gcontext, Box<dyn std::error::Error>> {
    let gc = conn.generate_id()?;
    conn.create_gc(
        gc,
        window,
        &CreateGCAux::new()
            .foreground(pixel_value)
            .background(if pixel_value > 128 { 0 } else { 255 })
            .line_width(1),
    )?;
    Ok(gc)
}

fn draw_chessboard(
    conn: &impl Connection,
    window: Window,
    black_gc: Gcontext,
    white_gc: Gcontext,
) -> Result<(), Box<dyn std::error::Error>> {
    for row in 0..8 {
        for col in 0..8 {
            let is_black = (row + col) % 2 == 0;
            let gc = if is_black { black_gc } else { white_gc };

            conn.poly_fill_rectangle(
                window,
                gc,
                &[Rectangle {
                    x: (col * SQUARE_SIZE) as i16,
                    y: (row * SQUARE_SIZE) as i16,
                    width: SQUARE_SIZE,
                    height: SQUARE_SIZE,
                }],
            )?;
        }
    }

    conn.poly_rectangle(
        window,
        black_gc,
        &[Rectangle {
            x: 0,
            y: 0,
            width: KINDLE_WIDTH - 1,
            height: (8 * SQUARE_SIZE) - 1,
        }],
    )?;

    Ok(())
}

fn draw_exit_zone(
    conn: &impl Connection,
    window: Window,
    gc: Gcontext,
) -> Result<(), Box<dyn std::error::Error>> {
    let y_start = (KINDLE_HEIGHT - EXIT_ZONE_HEIGHT) as i16;

    // Draw striped pattern to indicate exit zone
    for i in 0..5 {
        conn.poly_fill_rectangle(
            window,
            gc,
            &[Rectangle {
                x: 0,
                y: y_start + (i * 20),
                width: KINDLE_WIDTH,
                height: 10,
            }],
        )?;
    }

    // Draw border
    conn.poly_rectangle(
        window,
        gc,
        &[Rectangle {
            x: 0,
            y: y_start,
            width: KINDLE_WIDTH - 1,
            height: EXIT_ZONE_HEIGHT - 1,
        }],
    )?;

    // Draw "EXIT" indicator (series of rectangles)
    let indicators = [Rectangle {
        x: 486,
        y: y_start + 30,
        width: 100,
        height: 40,
    }];
    conn.poly_fill_rectangle(window, gc, &indicators)?;

    Ok(())
}

fn draw_touch_indicator(
    conn: &impl Connection,
    window: Window,
    gc: Gcontext,
    x: i16,
    y: i16,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.poly_arc(
        window,
        gc,
        &[Arc {
            x: x - 30,
            y: y - 30,
            width: 60,
            height: 60,
            angle1: 0,
            angle2: 360 * 64,
        }],
    )?;

    conn.poly_segment(
        window,
        gc,
        &[
            Segment {
                x1: x - 50,
                y1: y,
                x2: x + 50,
                y2: y,
            },
            Segment {
                x1: x,
                y1: y - 50,
                x2: x,
                y2: y + 50,
            },
        ],
    )?;

    Ok(())
}

fn draw_text_info(
    conn: &impl Connection,
    window: Window,
    gc: Gcontext,
    event_count: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let y_offset = (8 * SQUARE_SIZE + 10) as i16;

    let count = (event_count % 10) as usize;
    for i in 0..count {
        conn.poly_fill_rectangle(
            window,
            gc,
            &[Rectangle {
                x: (10 + i * 25) as i16,
                y: y_offset,
                width: 20,
                height: 20,
            }],
        )?;
    }

    Ok(())
}
