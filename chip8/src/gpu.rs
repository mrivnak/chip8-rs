pub const DISPLAY_HEIGHT: usize = 32;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_SIZE: usize = DISPLAY_HEIGHT * DISPLAY_WIDTH;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Pixel {
    On,
    Off,
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
