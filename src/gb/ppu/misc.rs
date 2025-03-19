/// Defines a Palette to colorize a Pixel
/// used by bgp, obp0 and obp1 registers.
pub struct Palette {
    map: [Color; 4],
}

impl Palette {
    pub fn colorize(&self, pixel: Pixel) -> Color {
        self.map[u8::from(pixel) as usize]
    }
}

impl From<u8> for Palette {
    /// Every two bits in the palette data byte represent a colour.
    /// Bits 7-6 maps to colour id 11, bits 5-4 map to colour id 10,
    /// bits 3-2 map to colour id 01 and bits 1-0 map to colour id 00
    fn from(value: u8) -> Self {
        Self {
            map: [
                Color::from(value & 0x3),
                Color::from((value >> 2) & 0x3),
                Color::from((value >> 4) & 0x3),
                Color::from((value >> 6) & 0x3),
            ],
        }
    }
}

/// Represents an non-colorized Pixel.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Pixel {
    Zero = 0x00,
    One = 0x01,
    Two = 0x10,
    Three = 0x11,
}

impl From<Pixel> for u8 {
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
    fn from(value: u8) -> Self {
        match value {
            0b00 => Pixel::Zero,
            0b01 => Pixel::One,
            0b10 => Pixel::Two,
            0b11 => Pixel::Three,
            _ => unimplemented!(),
        }
    }
}

/// Defines a colorized Pixel created
/// from a non-colorized Pixel with a Palette.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Color {
    White = 0x00,
    LightGrey = 0x01,
    DarkGrey = 0x10,
    Black = 0x11,
}

impl From<Color> for u8 {
    fn from(value: Color) -> u8 {
        match value {
            Color::White => 0b00,
            Color::LightGrey => 0b01,
            Color::DarkGrey => 0b10,
            Color::Black => 0b11,
        }
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Color::White,
            0b01 => Color::LightGrey,
            0b10 => Color::DarkGrey,
            0b11 => Color::Black,
            _ => unimplemented!(),
        }
    }
}
