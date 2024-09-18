use embedded_hal_async::spi::SpiBus;
use smart_leds::RGB8;

use crate::color_order::ColorOrder;
use crate::drivers::LedDriver;
use crate::encoding::encode_rgb8_to_spi_data;

/// WS2812 LED Driver supporting an arbitrary number of LEDs.
///
/// The caller is responsible for providing a buffer of appropriate size.
/// The buffer size should be:
/// `num_leds * 3 * 8` (8 bits for each color bit, encoded into a pulse stream)
/// plus sufficient bytes for the reset signal (based on SPI clock rate).
pub struct Ws2812<'a, SPI: SpiBus<u8>> {
    spi: SPI,
    color_order: ColorOrder,
    num_leds: usize,
    buffer: &'a mut [u8],
}

impl<'a, SPI: SpiBus<u8>> Ws2812<'a, SPI> {
    /// Creates a new WS2812 driver with the given SPI bus, number of LEDs, and buffer.
    ///
    /// # Arguments
    ///
    /// * `spi` - The SPI bus instance.
    /// * `num_leds` - The number of LEDs to control.
    /// * `buffer` - A mutable slice that must be large enough to hold the frame data.
    ///
    /// # Panics
    ///
    /// Panics if the provided buffer is too small.
    pub fn new(spi: SPI, num_leds: usize, buffer: &'a mut [u8]) -> Self {
        // WS2812 protocol uses 24 bits per LED (8 bits per RGB channel).
        // Each bit is encoded into 3 SPI bytes (1 byte = 8 pulses).
        let data_size = num_leds * 3 * 8; // 24 bits per LED -> 8 bits encoding per color.

        // Reset signal: 50 µs low.
        //  1 MHz SPI clock -> 50 µs needs at least 5 zero bytes.
        let reset_size = 5;

        let total_size = data_size + reset_size;

        assert!(
            buffer.len() >= total_size,
            "Buffer too small: required {}, provided {}",
            total_size,
            buffer.len()
        );

        Self {
            spi,
            color_order: ColorOrder::RGB,
            num_leds,
            buffer,
        }
    }

    /// Sets the color order if not RGB.
    pub fn set_color_order(&mut self, color_order: ColorOrder) {
        self.color_order = color_order;
    }
}

impl<'a, SPI: SpiBus<u8>> LedDriver<RGB8> for Ws2812<'a, SPI> {
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
        let data_size = num_leds * 3 * 8; // 24 bits per LED, encoded into 8 pulse bits per color
        let reset_size = 5; // Reset signal requires at least 50 µs low
        let total_size = data_size + reset_size;

        // Encode colors into the buffer using the timing-based pulse stream required by WS2812
        encode_rgb8_to_spi_data(
            &colors[..num_leds],
            self.color_order,
            &mut self.buffer[..data_size],
        );

        // Write the color data to SPI
        self.spi.write(&self.buffer[..data_size]).await?;

        // Write reset signal (ensure line is low for at least 50 µs)
        let reset_signal = &mut self.buffer[data_size..total_size];
        // Ensure the reset_signal is zeroed to hold the line low
        reset_signal.fill(0x00);
        self.spi.write(reset_signal).await
    }
}
