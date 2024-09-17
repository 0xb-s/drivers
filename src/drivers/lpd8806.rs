use crate::color_order::ColorOrder;
use crate::drivers::LedDriver;
use crate::encoding::encode_rgbw8_to_spi_data;
use crate::encoding::RGBW8;
use embedded_hal_async::spi::SpiBus;
use heapless::Vec;

pub struct Sk6812<SPI: SpiBus<u8>, const N: usize> {
    spi: SPI,
    color_order: ColorOrder,
    buffer: Vec<u8, N>,
}

impl<SPI: SpiBus<u8>, const N: usize> Sk6812<SPI, N> {
    /// Creates a new SK6812 driver with the given SPI bus.
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
        let total_size = num_leds * 16;

        // Resize the buffer based on the number of LEDs
        self.buffer.clear();
        self.buffer.resize(total_size, 0).ok();
    }
}

impl<SPI: SpiBus<u8>, const N: usize> LedDriver<RGBW8> for Sk6812<SPI, N> {
    type Error = SPI::Error;

    async fn write(&mut self, colors: &[RGBW8]) -> Result<(), Self::Error> {
        let num_leds = colors.len();

        // Initialize the buffer with the appropriate size based on the number of LEDs
        self.initialize_buffer(num_leds);

        // Encode colors into the buffer
        encode_rgbw8_to_spi_data(&colors[..num_leds], self.color_order, &mut self.buffer[..]);

        // Write the data to SPI
        self.spi.write(&self.buffer[..]).await?;

        // Write reset signal (fixed size)
        let reset_signal = [0_u8; 140];
        self.spi.write(&reset_signal).await
    }
}
