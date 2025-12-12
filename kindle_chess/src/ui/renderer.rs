use crate::events::Rectangle;
use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc as StdArc;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, *};
use x11rb::COPY_DEPTH_FROM_PARENT; // Rename to avoid collision

pub struct Renderer {
    conn: StdArc<x11rb::rust_connection::RustConnection>,
    screen_num: usize,
    window: Window,
    gcs: HashMap<DrawColor, Gcontext>,
    dirty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawColor {
    Black,
    White,
    Gray,
    DarkGray,
    LightGray,
}

impl Renderer {
    pub fn new(
    ) -> Result<(Self, StdArc<x11rb::rust_connection::RustConnection>), Box<dyn std::error::Error>>
    {
        // Connect to X11
        let (conn, screen_num) = match x11rb::connect(Some(":0.0")) {
            Ok(result) => result,
            Err(_) => {
                warn!("Failed to connect to :0.0, trying :0");
                x11rb::connect(Some(":0"))?
            }
        };

        // Wrap connection in Arc for sharing
        let conn = StdArc::new(conn);

        info!("Connected to X11, screen number: {}", screen_num);

        let screen = &conn.setup().roots[screen_num];

        // Create window
        let window = conn.generate_id()?;

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
            window,
            screen.root,
            0,
            0,
            1072,
            1448,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &win_aux,
        )?;

        // Create GCs
        let mut gcs = HashMap::new();

        for (color, pixel_value) in [
            (DrawColor::Black, 0),
            (DrawColor::White, 255),
            (DrawColor::Gray, 128),
            (DrawColor::DarkGray, 64),
            (DrawColor::LightGray, 192),
        ] {
            let gc = conn.generate_id()?;
            conn.create_gc(
                gc,
                window,
                &CreateGCAux::new()
                    .foreground(pixel_value)
                    .background(if pixel_value > 128 { 0 } else { 255 }),
            )?;
            gcs.insert(color, gc);
        }

        // Map window
        conn.map_window(window)?;
        conn.configure_window(
            window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        )?;
        conn.flush()?;

        let renderer = Self {
            conn: conn.clone(), // Clone the Arc
            screen_num,
            window,
            gcs,
            dirty: true,
        };

        Ok((renderer, conn)) // Return both renderer and connection Arc
    }

    pub fn draw_rectangle(
        &mut self,
        rect: Rectangle,
        color: DrawColor,
        filled: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let gc = self.gcs[&color];

        if filled {
            self.conn.poly_fill_rectangle(
                self.window,
                gc,
                &[xproto::Rectangle {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                }],
            )?;
        } else {
            self.conn.poly_rectangle(
                self.window,
                gc,
                &[xproto::Rectangle {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                }],
            )?;
        }

        self.dirty = true;
        Ok(())
    }

    pub fn draw_circle(
        &mut self,
        center_x: i16,
        center_y: i16,
        radius: u16,
        color: DrawColor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let gc = self.gcs[&color];

        // Use the X11 Arc type explicitly
        self.conn.poly_arc(
            self.window,
            gc,
            &[xproto::Arc {
                // Explicitly use xproto::Arc
                x: center_x - radius as i16,
                y: center_y - radius as i16,
                width: radius * 2,
                height: radius * 2,
                angle1: 0,
                angle2: 360 * 64,
            }],
        )?;

        self.dirty = true;
        Ok(())
    }

    pub fn draw_line(
        &mut self,
        x1: i16,
        y1: i16,
        x2: i16,
        y2: i16,
        color: DrawColor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let gc = self.gcs[&color];

        self.conn
            .poly_segment(self.window, gc, &[Segment { x1, y1, x2, y2 }])?;

        self.dirty = true;
        Ok(())
    }

    pub fn clear(&mut self, color: DrawColor) -> Result<(), Box<dyn std::error::Error>> {
        self.draw_rectangle(Rectangle::new(0, 0, 1072, 1448), color, true)
    }

    pub fn present(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.dirty {
            self.conn.flush()?;
            self.dirty = false;
        }
        Ok(())
    }

    pub fn window(&self) -> Window {
        self.window
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = self.conn.destroy_window(self.window);
        let _ = self.conn.flush();
    }
}
