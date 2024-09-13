use embedded_hal_async::spi::SpiBus;
use smart_leds::RGB8;

use crate::color_order::ColorOrder;
use crate::drivers::LedDriver;
use crate::encoding::encode_rgb8_to_spi_data;
const MAX_LEDS: usize = 60;
pub struct Ws2812<SPI: SpiBus<u8>> {
    spi: SPI,
    color_order: ColorOrder,
    buffer: [u8; MAX_BUFFER_SIZE],
}

const MAX_BUFFER_SIZE: usize = MAX_LEDS * 12; // Each LED requires 12 bytes

impl<SPI: SpiBus<u8>> Ws2812<SPI> {
    /// Creates a new WS2812 driver with the given SPI bus.
    /// Colors default to RGB order.
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            color_order: ColorOrder::RGB,
            buffer: [0; MAX_BUFFER_SIZE],
        }
    }

    /// Sets the color order if not RGB.
    pub fn set_color_order(&mut self, color_order: ColorOrder) {
        self.color_order = color_order;
    }
}

impl<SPI: SpiBus<u8>> LedDriver<RGB8> for Ws2812<SPI> {
    type Error = SPI::Error;

    async fn write(&mut self, colors: &[RGB8]) -> Result<(), Self::Error> {
        let num_leds = core::cmp::min(colors.len(), MAX_LEDS);
        let data_size = num_leds * 12;

        // Encode colors into the buffer
        encode_rgb8_to_spi_data(
            &colors[..num_leds],
            self.color_order,
            &mut self.buffer[..data_size],
        );

        // Write the data to SPI
        self.spi.write(&self.buffer[..data_size]).await?;

        // Write reset signal.
        let reset_signal = [0_u8; 140];
        self.spi.write(&reset_signal).await
    }
}
