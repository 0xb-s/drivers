use crate::color_order::ColorOrder;
use smart_leds::RGB8;

pub const PATTERNS: [u8; 4] = [0b1000_1000, 0b1000_1110, 0b1110_1000, 0b1110_1110];

/// Encodes RGB8 data into SPI data buffer.
pub fn encode_rgb8_to_spi_data(colors: &[RGB8], color_order: ColorOrder, data: &mut [u8]) {
    let mut offset = 0;
    for RGB8 { r, g, b } in colors {
        let color_values = match color_order {
            ColorOrder::RGB => [*r, *g, *b],
            ColorOrder::GRB => [*g, *r, *b],
        };
        for &color in &color_values {
            let mut color = color;
            for _ in 0..4 {
                data[offset] = PATTERNS[((color & 0b1100_0000) >> 6) as usize];
                color <<= 2;
                offset += 1;
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RGBW8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub w: u8,
}

impl RGBW8 {
    /// Creates a new `RGBW8` color.
    pub const fn new(r: u8, g: u8, b: u8, w: u8) -> Self {
        RGBW8 { r, g, b, w }
    }
}
pub fn encode_rgbw8_to_spi_data(colors: &[RGBW8], color_order: ColorOrder, data: &mut [u8]) {
    // Each RGBW LED requires 16 bytes in the data buffer
    let required_size = colors.len() * 16;
    assert!(
        data.len() >= required_size,
        "Data buffer too small; expected at least {} bytes, got {} bytes",
        required_size,
        data.len()
    );

    let mut offset = 0;

    for &RGBW8 { r, g, b, w } in colors {
        // Arrange the color components according to the specified color order
        let color_values = match color_order {
            ColorOrder::RGB => [r, g, b, w],
            ColorOrder::GRB => [g, r, b, w],
        };

        // Encode each color component
        for &color_byte in &color_values {
            let mut color = color_byte;

            // Each color component is encoded into 4 SPI bytes
            for _ in 0..4 {
                // Extract the top 2 bits of the color byte
                let bits = (color & 0b1100_0000) >> 6;
                data[offset] = PATTERNS[bits as usize];
                color <<= 2; // Shift left to process the next 2 bits
                offset += 1;
            }
        }
    }
}
