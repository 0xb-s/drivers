use crate::color_order::ColorOrder;
use crate::drivers::LedDriver;
use crate::encoding::encode_rgbw8_to_spi_data;
use crate::encoding::RGBW8;
use embedded_hal_async::spi::SpiBus;

/// SK6812 LED Driver supporting an arbitrary number of LEDs.
///
/// The caller is responsible for providing a buffer of appropriate size.
/// The buffer size should be:
/// `data_size + reset_size`,
/// where:
/// - `data_size` = `num_leds * 4` (4 bytes per LED for RGBW encoding)
/// - `reset_size` = Sufficient size for the reset signal based on SPI timing requirements
pub struct Sk6812<'a, SPI: SpiBus<u8>> {
    spi: SPI,
    color_order: ColorOrder,
    num_leds: usize,
    buffer: &'a mut [u8],
}

impl<'a, SPI: SpiBus<u8>> Sk6812<'a, SPI> {
    /// Creates a new SK6812 driver with the given SPI bus, number of LEDs, and buffer.
    ///
    /// # Arguments
    ///
    /// * `spi` - The SPI bus instance.
    /// * `num_leds` - The number of LEDs to control.
    /// * `buffer` - A mutable slice that must be large enough to hold all frames.
    ///
    /// # Panics
    ///
    /// Panics if the provided buffer is too small.
    pub fn new(spi: SPI, num_leds: usize, buffer: &'a mut [u8]) -> Self {
        // Each LED requires 4 bytes (RGBW).
        let data_size = num_leds * 4;
        // Reset signal: At least 80 microseconds low. Assuming a 1 MHz clock, this is about 10 bytes.
        let reset_size = 10;
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

impl<'a, SPI: SpiBus<u8>> LedDriver<RGBW8> for Sk6812<'a, SPI> {
    type Error = SPI::Error;

    async fn write(&mut self, colors: &[RGBW8]) -> Result<(), Self::Error> {
        let num_leds = core::cmp::min(colors.len(), self.num_leds);
        let data_size = num_leds * 4; // 4 bytes per LED for RGBW encoding
        let reset_size = 10; // (80µs reset signal)
        let total_size = data_size + reset_size;

        // Encode colors into the buffer
        encode_rgbw8_to_spi_data(
            &colors[..num_leds],
            self.color_order,
            &mut self.buffer[..data_size],
        );

        // Write the color data to SPI
        self.spi.write(&self.buffer[..data_size]).await?;

        // Write reset signal (at least 80µs low, which can be achieved by sending a few zero bytes)
        let reset_signal = &mut self.buffer[data_size..total_size];
        reset_signal.fill(0x00); // Reset signal is just zeros
        self.spi.write(reset_signal).await
    }
}
