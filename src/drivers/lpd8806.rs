use crate::drivers::LedDriver;
use embedded_hal_async::spi::SpiBus;
use smart_leds::RGB8;

/// LPD8806 LED Driver supporting an arbitrary number of LEDs.
///
/// The caller is responsible for providing a buffer of appropriate size.
/// The buffer size should be:
/// `start_frame_size + (num_leds * led_frame_size) + end_frame_size`,
/// where:
/// - `start_frame_size` = 4 bytes (always zero).
/// - `led_frame_size` = 3 bytes per LED (each color component is 7 bits, MSB is always 1).
/// - `end_frame_size` = `(num_leds + 31) / 32` bytes.
pub struct Lpd8806<'a, SPI: SpiBus<u8>> {
    spi: SPI,
    num_leds: usize,
    buffer: &'a mut [u8],
}

impl<'a, SPI: SpiBus<u8>> Lpd8806<'a, SPI> {
    /// Creates a new LPD8806 driver with the given SPI bus, number of LEDs, and buffer.
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
        let led_frame_size = 3;
        let end_frame_size = (num_leds + 31) / 32;
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

impl<'a, SPI: SpiBus<u8>> LedDriver<RGB8> for Lpd8806<'a, SPI> {
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
        let end_frame_size = (num_leds + 31) / 32;

        // Frame sizes
        let start_frame_size = 4;
        let led_frame_size = 3;
        let total_size = start_frame_size + (num_leds * led_frame_size) + end_frame_size;

        // Start frame: all zeros
        self.buffer[..start_frame_size].fill(0x00);

        // LED frames
        for (i, &RGB8 { r, g, b }) in colors.iter().enumerate().take(num_leds) {
            let offset = start_frame_size + i * led_frame_size;
            // Each color component is 7 bits with the highest bit set to 1.
            // This is specific to the LPD8806 protocol.
            self.buffer[offset] = (g >> 1) | 0x80; // Green channel (7 bits, MSB set to 1)
            self.buffer[offset + 1] = (r >> 1) | 0x80; // Red channel (7 bits, MSB set to 1)
            self.buffer[offset + 2] = (b >> 1) | 0x80; // Blue channel (7 bits, MSB set to 1)
        }

        // End frame: all zeros
        let end_frame_offset = start_frame_size + num_leds * led_frame_size;
        self.buffer[end_frame_offset..end_frame_offset + end_frame_size].fill(0x00);

        // Write the data via SPI
        self.spi.write(&self.buffer[..total_size]).await
    }
}
