use crate::ui::events::RectangleExt;
use image::{ImageBuffer, Luma, imageops};
use log::info;
use std::collections::HashMap;
use std::sync::Arc as StdArc;
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, *};

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
    pub fn new()
    -> Result<(Self, StdArc<x11rb::rust_connection::RustConnection>), Box<dyn std::error::Error>>
    {
        // Connect to X11
        let (conn, screen_num) = x11rb::connect(None)?;

        // Wrap connection in Arc for sharing
        let conn = StdArc::new(conn);

        info!("Connected to X11, screen number: {}", screen_num);

        let screen = &conn.setup().roots[screen_num];

        // Create window
        let window = conn.generate_id()?;

        let win_aux = CreateWindowAux::new()
            .background_pixel(screen.white_pixel)
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
            self.conn.poly_fill_rectangle(self.window, gc, &[rect])?;
        } else {
            self.conn.poly_rectangle(self.window, gc, &[rect])?;
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

    pub fn draw_image(
        &mut self,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        img: &ImageBuffer<Luma<u8>, Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use image::imageops::FilterType;
        use x11rb::protocol::xproto::ImageFormat;

        let scaled = imageops::resize(img, width as u32, height as u32, FilterType::Nearest);

        let screen = &self.conn.setup().roots[self.screen_num];
        let depth = screen.root_depth;

        // Each pixel must match the drawable depth:
        //   8-bit  → 1 byte per pixel
        //   16-bit → 2 bytes per pixel
        //   24/32  → 4 bytes per pixel (BGRX, scanlines still padded to 4 bytes)
        let bytes_per_pixel: usize = match depth {
            8 => 1,
            16 => 2,
            _ => 4, // 24 or 32-bit — the common dev-machine case
        };

        let row_bytes = width as usize * bytes_per_pixel;
        let width_padded = ((row_bytes + 3) / 4) * 4;
        let mut data: Vec<u8> = Vec::with_capacity(width_padded * height as usize);

        for row in scaled.rows() {
            let mut row_data: Vec<u8> = Vec::with_capacity(row_bytes);
            for px in row {
                let v = px[0];
                match bytes_per_pixel {
                    1 => row_data.push(v),
                    2 => {
                        row_data.push(v);
                        row_data.push(v);
                    }
                    _ => {
                        row_data.push(v);
                        row_data.push(v);
                        row_data.push(v);
                        row_data.push(0);
                    }
                }
            }
            // Pad scanline to 4-byte boundary
            row_data.resize(width_padded, 0);
            data.extend_from_slice(&row_data);
        }

        self.conn.put_image(
            ImageFormat::Z_PIXMAP,
            self.window,
            self.gcs[&DrawColor::Black],
            width,
            height,
            x,
            y,
            0,     // left_pad
            depth, // use actual drawable depth, not hardcoded 8
            &data,
        )?;

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
