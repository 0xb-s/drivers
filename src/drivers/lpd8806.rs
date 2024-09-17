use embedded_hal_async::spi::SpiBus;
use heapless::Vec;
use smart_leds::RGB8;

use crate::color_order::ColorOrder;
use crate::drivers::LedDriver;
use crate::encoding::encode_rgb8_to_spi_data;

pub struct Ws2812<SPI: SpiBus<u8>, const N: usize> {
    spi: SPI,
    color_order: ColorOrder,
    buffer: Vec<u8, N>,
}

impl<SPI: SpiBus<u8>, const N: usize> Ws2812<SPI, N> {
    /// Creates a new WS2812 driver with the given SPI bus.
    /// Colors default to RGB order.
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            color_order: ColorOrder::RGB,
            buffer: Vec::new(),
        }
    }

    /// Sets the color order if not RGB.
    pub fn set_color_order(&mut self, color_order: ColorOrder) {
        self.color_order = color_order;
    }

    /// Initializes the buffer with the required size for the given number of LEDs.
    fn initialize_buffer(&mut self, num_leds: usize) {
        let total_size = num_leds * 12; // LED requires 12 bytes to represent its RGB data

        // Resize the buffer based on the number of LEDs
        self.buffer.clear();
        self.buffer.resize(total_size, 0).ok(); // Resize the buffer, handle any error silently
    }
}

impl<SPI: SpiBus<u8>, const N: usize> LedDriver<RGB8> for Ws2812<SPI, N> {
    type Error = SPI::Error;

    async fn write(&mut self, colors: &[RGB8]) -> Result<(), Self::Error> {
        let num_leds = colors.len();

        // Initialize the buffer with the appropriate size based on the number of LEDs
        self.initialize_buffer(num_leds);

        // Encode colors into the buffer
        encode_rgb8_to_spi_data(&colors[..num_leds], self.color_order, &mut self.buffer[..]);

        // Write the data to SPI
        self.spi.write(&self.buffer[..]).await?;

        // Write reset signal.
        let reset_signal = [0_u8; 140];
        self.spi.write(&reset_signal).await
    }
}
