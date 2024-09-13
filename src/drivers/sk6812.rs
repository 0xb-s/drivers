use crate::color_order::ColorOrder;
use crate::drivers::LedDriver;
use crate::encoding::encode_rgbw8_to_spi_data;
use crate::encoding::RGBW8;
use embedded_hal_async::spi::SpiBus;

pub struct Sk6812<SPI: SpiBus<u8>> {
    spi: SPI,
    color_order: ColorOrder,
    buffer: [u8; MAX_BUFFER_SIZE],
}

const MAX_LEDS: usize = 60;
const MAX_BUFFER_SIZE: usize = MAX_LEDS * 16;

impl<SPI: SpiBus<u8>> Sk6812<SPI> {
    /// Creates a new SK6812 driver with the given SPI bus.
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

impl<SPI: SpiBus<u8>> LedDriver<RGBW8> for Sk6812<SPI> {
    type Error = SPI::Error;

    async fn write(&mut self, colors: &[RGBW8]) -> Result<(), Self::Error> {
        let num_leds = core::cmp::min(colors.len(), MAX_LEDS);
        let data_size = num_leds * 16;

        // Encode colors into the buffer
        encode_rgbw8_to_spi_data(
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
