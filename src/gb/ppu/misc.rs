/// Defines a Palette to colorize a Pixel
/// used by bgp, obp0 and obp1 registers.
#[derive(Default, Copy, Clone)]
pub struct Palette {
    map: [ColoredPixel; 4],
}

impl Palette {
    #[inline]
    pub fn colorize(&self, pixel: Pixel) -> ColoredPixel {
        self.map[u8::from(pixel) as usize]
    }
}

impl From<u8> for Palette {
    /// Every two bits in the palette data byte represent a colour.
    /// Bits 7-6 maps to colour id 11, bits 5-4 map to colour id 10,
    /// bits 3-2 map to colour id 01 and bits 1-0 map to colour id 00
    #[inline]
    fn from(value: u8) -> Self {
        Self {
            map: [
                ColoredPixel::from(value & 0b11),
                ColoredPixel::from((value >> 2) & 0b11),
                ColoredPixel::from((value >> 4) & 0b11),
                ColoredPixel::from((value >> 6) & 0b11),
            ],
        }
    }
}

impl From<Palette> for u8 {
    #[inline]
    fn from(palette: Palette) -> u8 {
        let mut value = 0;
        value |= u8::from(palette.map[0]);
        value |= u8::from(palette.map[1]) << 2;
        value |= u8::from(palette.map[2]) << 4;
        value |= u8::from(palette.map[3]) << 6;
        value
    }
}

/// Represents an non-colorized Pixel.
#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub enum Pixel {
    #[default]
    Zero,
    One,
    Two,
    Three,
}

impl From<Pixel> for u8 {
    #[inline]
    fn from(value: Pixel) -> u8 {
        match value {
            Pixel::Zero => 0b00,
            Pixel::One => 0b01,
            Pixel::Two => 0b10,
            Pixel::Three => 0b11,
        }
    }
}

impl From<u8> for Pixel {
    #[inline]
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => Pixel::Zero,
            0b01 => Pixel::One,
            0b10 => Pixel::Two,
            0b11 => Pixel::Three,
            _ => unreachable!(),
        }
    }
}

/// Defines a colorized Pixel created from a non-colorized Pixel with a Palette.
#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub enum ColoredPixel {
    #[default]
    White,
    LightGrey,
    DarkGrey,
    Black,
}

impl From<ColoredPixel> for u8 {
    #[inline]
    fn from(value: ColoredPixel) -> u8 {
        match value {
            ColoredPixel::White => 0b00,
            ColoredPixel::LightGrey => 0b01,
            ColoredPixel::DarkGrey => 0b10,
            ColoredPixel::Black => 0b11,
        }
    }
}

impl From<u8> for ColoredPixel {
    #[inline]
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => ColoredPixel::White,
            0b01 => ColoredPixel::LightGrey,
            0b10 => ColoredPixel::DarkGrey,
            0b11 => ColoredPixel::Black,
            _ => unreachable!(),
        }
    }
}

/// Represents a Sprite in the OAM memory.
#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    pub x: u8, // Object’s horizontal position on the screen + 8
    pub y: u8, // Object’s vertical position on the screen + 16
    pub tile_index: u8,
    pub attrs: SpriteAttributes,
}

impl Sprite {
    #[inline]
    pub const fn new(y: u8, x: u8, tile_index: u8, attrs: SpriteAttributes) -> Self {
        Self {
            x,
            y,
            tile_index,
            attrs,
        }
    }
}

bitflags! {
    /// Represents the attributes of a sprite.
    /// The first 3 bits are only used in CGB mode.
    /// Bit 4 is used to select the palette in CGB mode.
    #[derive(Debug, Copy, Clone)]
    pub struct SpriteAttributes: u8 {
        const BANK          = 0b0000_1000;
        const DMG_PALETTE   = 0b0001_0000;
        const X_FLIP        = 0b0010_0000;
        const Y_FLIP        = 0b0100_0000;
        const PRIORITY      = 0b1000_0000;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_palette() {
        let palette = Palette::from(0b11_10_01_00);
        assert_eq!(palette.colorize(Pixel::Zero), ColoredPixel::White);
        assert_eq!(palette.colorize(Pixel::One), ColoredPixel::LightGrey);
        assert_eq!(palette.colorize(Pixel::Two), ColoredPixel::DarkGrey);
        assert_eq!(palette.colorize(Pixel::Three), ColoredPixel::Black);
        assert_eq!(u8::from(palette), 0b11_10_01_00);
    }

    #[test]
    fn test_pixel() {
        let data = vec![
            (0b00, Pixel::Zero),
            (0b01, Pixel::One),
            (0b10, Pixel::Two),
            (0b11, Pixel::Three),
        ];
        for (value, pixel) in data {
            assert_eq!(u8::from(pixel), value);
            assert_eq!(Pixel::from(value), pixel);
        }

        assert_eq!(Pixel::from(0b1111_1111), Pixel::Three);
        assert_eq!(Pixel::from(0b0101_1100), Pixel::Zero);
    }

    #[test]
    fn test_colored_pixel() {
        let data = vec![
            (0b00, ColoredPixel::White),
            (0b01, ColoredPixel::LightGrey),
            (0b10, ColoredPixel::DarkGrey),
            (0b11, ColoredPixel::Black),
        ];
        for (value, pixel) in data {
            assert_eq!(u8::from(pixel), value);
            assert_eq!(ColoredPixel::from(value), pixel);
        }

        assert_eq!(ColoredPixel::from(0b1111_1111), ColoredPixel::Black);
        assert_eq!(ColoredPixel::from(0b0101_1100), ColoredPixel::White);
    }
}
