use embedded_hal_async::spi::SpiBus;
use smart_leds::RGB8;

use crate::drivers::LedDriver;

/// APA102 LED Driver supporting arbitrary number of LEDs.
///
/// The caller is responsible for providing a buffer of appropriate size.
pub struct Apa102<'a, SPI: SpiBus<u8>> {
    spi: SPI,
    num_leds: usize,
    buffer: &'a mut [u8],
}

impl<'a, SPI: SpiBus<u8>> Apa102<'a, SPI> {
    /// Creates a new APA102 driver with the given SPI bus, number of LEDs, and buffer.
    /// Panics if the provided buffer is too small.
    pub fn new(spi: SPI, num_leds: usize, buffer: &'a mut [u8]) -> Self {
        let start_frame_size = 4;
        let led_frame_size = 4;
        let end_frame_size = (num_leds + 15) / 16;
        let total_size = start_frame_size + (num_leds * led_frame_size) + end_frame_size;

        assert!(
            buffer.len() >= total_size,
            "Buffer too small: required {}, provided {}",
            total_size,
            buffer.len()
        );

        Self {
            spi,
            num_leds,
            buffer,
        }
    }
}

impl<'a, SPI: SpiBus<u8>> LedDriver<RGB8> for Apa102<'a, SPI> {
    type Error = SPI::Error;

    async fn write(&mut self, colors: &[RGB8]) -> Result<(), Self::Error> {
        let num_leds = core::cmp::min(colors.len(), self.num_leds);
        let end_frame_size = (num_leds + 15) / 16;

        // Frame sizes
        let start_frame_size = 4;
        let led_frame_size = 4;
        let total_size = start_frame_size + (num_leds * led_frame_size) + end_frame_size;

        // Start frame: all zeros
        self.buffer[..start_frame_size].fill(0x00);

        // LED frames
        for (i, &RGB8 { r, g, b }) in colors.iter().enumerate().take(num_leds) {
            let offset = start_frame_size + i * led_frame_size;
            // Data frame: 0xE0 | brightness (set to max 0x1F), then B, G, R.
            self.buffer[offset] = 0xE0 | 0x1F; // Global brightness set to max (0x1F)
            self.buffer[offset + 1] = b;
            self.buffer[offset + 2] = g;
            self.buffer[offset + 3] = r;
        }

        // End frame: all zeros
        let end_frame_offset = start_frame_size + num_leds * led_frame_size;
        self.buffer[end_frame_offset..end_frame_offset + end_frame_size].fill(0x00);

        // Write the data via SPI
        self.spi.write(&self.buffer[..total_size]).await
    }
}
