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

    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        for (j, &byte) in sprite.iter().enumerate() {
            for i in 0..8 {
                let x = (x + i) % DISPLAY_WIDTH;
                let y = (y + j) % DISPLAY_HEIGHT;
                let pixel = (byte >> (7 - i)) & 1;
                let index = y * DISPLAY_WIDTH + x;
                let old_pixel = self.pixels[index];
                let new_pixel = xor_pixel(old_pixel, pixel);

                self.pixels[index] = new_pixel;
                if old_pixel == Pixel::On && new_pixel == Pixel::Off {
                    collision = true;
                }
            }
        }
        collision
    }
}

fn xor_pixel(old: Pixel, new: u8) -> Pixel {
    debug_assert!(new < 2);
    match (old, new) {
        (Pixel::On, 1) => Pixel::Off,
        (Pixel::On, 0) => Pixel::On,
        (Pixel::Off, 1) => Pixel::On,
        (Pixel::Off, 0) => Pixel::Off,
        _ => unreachable!(),
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

    #[test]
    fn test_xor_pixel() {
        assert_eq!(xor_pixel(Pixel::On, 1), Pixel::Off);
        assert_eq!(xor_pixel(Pixel::On, 0), Pixel::On);
        assert_eq!(xor_pixel(Pixel::Off, 1), Pixel::On);
        assert_eq!(xor_pixel(Pixel::Off, 0), Pixel::Off);
    }

    #[test]
    #[should_panic]
    fn test_xor_pixel_invalid() {
        xor_pixel(Pixel::On, 2);
    }

    #[test]
    fn test_draw() {
        let mut display = Display::default();
        let sprite = [0b11110000, 0b00001111];
        let collision = display.draw(0, 0, &sprite);
        let on_pixels = [
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 1),
            (5, 1),
            (6, 1),
            (7, 1),
        ];
        for &(x, y) in on_pixels.iter() {
            assert_eq!(display.pixels[y * DISPLAY_WIDTH + x], Pixel::On);
        }
        assert!(!collision);

        let collision = display.draw(0, 0, &sprite);
        let on_pixels = [
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 1),
            (5, 1),
            (6, 1),
            (7, 1),
        ];
        for &(x, y) in on_pixels.iter() {
            assert_eq!(display.pixels[y * DISPLAY_WIDTH + x], Pixel::Off);
        }
        assert!(collision);
    }
}
