use embedded_graphics::{
    Pixel,
    pixelcolor::Gray8,
    prelude::{Dimensions, DrawTarget, GrayColor, OriginDimensions, PointsIter, Size},
};

/// SPI communication error
#[derive(Debug)]
struct CommError;

/// Representation of the Kindle Display.
pub struct KindleDisplay {
    /// The framebuffer with one `u8` value per pixel.
    pub framebuffer: &'static mut [u8],

    /// The interface to the display controller.
    //pub iface: SPI1,
    pub width: u32,
    pub height: u32,
    pub stride: u32,

    pub rotation: u8,
}

pub enum RefreshMode {
    Full,
    Partial,
    Fast,
}

impl KindleDisplay {
    pub fn new_test() -> Self {
        let mut buffer = vec![0u8; 1072 * 1448].into_boxed_slice();
        let framebuffer = Box::leak(buffer);

        KindleDisplay {
            framebuffer: framebuffer,
            width: 1072,
            height: 1448,
            stride: 1072,
            rotation: 3,
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(path, self.framebuffer as &[u8])?;
        Ok(())
    }
    // Updates the display from the framebuffer.
    // TODO: This will need a special implementation with printing the negative of the thing that's to be deleted from the screen
    pub fn flush(&self, mode: RefreshMode) -> Result<(), Box<dyn std::error::Error>> {
        //let mut eips = std::process::Command::new("eips");
        let refresh_status = match mode {
            RefreshMode::Full => Ok((self.save_to_file("./display_test.raw"))),
            RefreshMode::Partial => Ok(()),
            RefreshMode::Fast => Ok(()),
        };

        refresh_status
    }
}

impl DrawTarget for KindleDisplay {
    type Color = Gray8;
    // Fits / transforms the image information to the actual hardware specs
    // == coordinates from logical space to physical framebuffer memory space
    //
    // `KindleDisplay` uses a framebuffer and doesn't need to communicate with the display
    // controller to draw pixel, which means that drawing operations can never fail. To reflect
    // this the type `Infallible` was chosen as the `Error` type.
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            // Check if the pixel coordinates are out of bounds.
            if let Ok((x @ 0..=1071, y @ 0..=1447)) = coord.try_into() {
                // Rotate
                let physical_x = y;
                let physical_y = self.width - 1 - x;

                // Calculate the index in the framebuffer.
                let index: u32 = physical_x + physical_y * self.stride;

                // Write pixel.
                self.framebuffer[index as usize] = color.luma();
            }
        }

        Ok(())
    }

    fn fill_contiguous<I>(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.draw_iter(
            area.points()
                .zip(colors)
                .map(|(pos, color)| embedded_graphics::Pixel(pos, color)),
        )
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        self.fill_contiguous(area, core::iter::repeat(color))
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_solid(&self.bounding_box(), color)
    }
}

impl OriginDimensions for KindleDisplay {
    fn size(&self) -> Size {
        Size::new(1072, 1448)
    }
}
