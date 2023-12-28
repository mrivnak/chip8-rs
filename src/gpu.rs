pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_SIZE: usize = DISPLAY_HEIGHT * DISPLAY_WIDTH;

const PIXEL_COLOR_ON: (u8, u8, u8, u8) = (0xFF, 0xFF, 0xFF, 0xFF);
const PIXEL_COLOR_OFF: (u8, u8, u8, u8) = (0x00, 0x00, 0x00, 0xFF);

#[derive(Copy, Clone)]
pub enum Pixel {
    On,
    Off,
}

impl From<Pixel> for (u8, u8, u8, u8) {
    fn from(pixel: Pixel) -> Self {
        match pixel {
            Pixel::On => PIXEL_COLOR_ON,
            Pixel::Off => PIXEL_COLOR_OFF,
        }
    }
}

pub struct Display {
    pub pixels: [Pixel; DISPLAY_SIZE],
}

impl Default for Display {
    fn default() -> Display {
        Display {
            pixels: [Pixel::Off; DISPLAY_SIZE],
        }
    }
}

impl Display {
    pub fn clear(&mut self) {
        self.pixels = [Pixel::Off; DISPLAY_SIZE];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear() {
        let mut display = Display::default();
        display.pixels = [Pixel::On; DISPLAY_SIZE];
        display.clear();
        assert_eq!(display.pixels, [Pixel::Off; DISPLAY_SIZE]);
    }
}
