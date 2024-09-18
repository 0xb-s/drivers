use embedded_hal_async::spi::SpiBus;
use smart_leds::RGB8;
use crate::drivers::LedDriver;

/// APA102 LED Driver supporting an arbitrary number of LEDs.
///
/// The caller is responsible for providing a buffer of appropriate size.
/// The buffer size should be:
/// `start_frame_size + (num_leds * led_frame_size) + end_frame_size`,
/// where:
/// - `start_frame_size` = 4 bytes (always zero).
/// - `led_frame_size` = 4 bytes per LED (1 for brightness, 3 for RGB data).
/// - `end_frame_size` = `(num_leds + 15) / 16` bytes.
pub struct Apa102<'a, SPI: SpiBus<u8>> {
    spi: SPI,
    num_leds: usize,
    buffer: &'a mut [u8],
}

impl<'a, SPI: SpiBus<u8>> Apa102<'a, SPI> {
    /// Creates a new APA102 driver with the given SPI bus, number of LEDs, and buffer.
    ///
    /// # Arguments
    ///
    /// * `spi` - The SPI bus instance.
    /// * `num_leds` - The number of LEDs to control.
    /// * `buffer` - A mutable slice that must be large enough to hold the frame data.
    ///
    /// # Panics
    ///
    /// This function will panic if the provided buffer is too small to hold all frame data.
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

    /// Writes the RGB data to the LED strip.
    ///
    /// # Arguments
    ///
    /// * `colors` - A slice of RGB8 colors to write to the strip. This function will write
    ///   up to the number of LEDs configured at creation time (`num_leds`).
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating whether the write was successful.
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
            // Data frame: 0xE0 | brightness (set to max 0x1F), followed by B, G, R.
            self.buffer[offset] = 0xE0 | 0x1F; // Global brightness set to max (0x1F)
            self.buffer[offset + 1] = b; // Blue channel
            self.buffer[offset + 2] = g; // Green channel
            self.buffer[offset + 3] = r; // Red channel
        }

        // End frame: all zeros
        let end_frame_offset = start_frame_size + num_leds * led_frame_size;
        self.buffer[end_frame_offset..end_frame_offset + end_frame_size].fill(0x00);

        // Write the data via SPI
        self.spi.write(&self.buffer[..total_size]).await
    }
}
