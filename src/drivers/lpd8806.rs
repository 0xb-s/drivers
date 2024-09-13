use crate::drivers::LedDriver;
use embedded_hal_async::spi::SpiBus;
use smart_leds::RGB8;

pub struct Lpd8806<SPI: SpiBus<u8>> {
    spi: SPI,
    buffer: [u8; MAX_BUFFER_SIZE],
}

const MAX_LEDS: usize = 60; // Adjust as needed
const START_FRAME_SIZE: usize = 4;
const LED_FRAME_SIZE: usize = 3;
const END_FRAME_SIZE: usize = (MAX_LEDS + 31) / 32;
const MAX_BUFFER_SIZE: usize = START_FRAME_SIZE + (MAX_LEDS * LED_FRAME_SIZE) + END_FRAME_SIZE;

impl<SPI: SpiBus<u8>> Lpd8806<SPI> {
    /// Creates a new LPD8806 driver with the given SPI bus.
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            buffer: [0; MAX_BUFFER_SIZE],
        }
    }
}

impl<SPI: SpiBus<u8>> LedDriver<RGB8> for Lpd8806<SPI> {
    type Error = SPI::Error;

    async fn write(&mut self, colors: &[RGB8]) -> Result<(), Self::Error> {
        let num_leds = core::cmp::min(colors.len(), MAX_LEDS);

        // Prepare the buffer.
        let buffer = &mut self.buffer[..];

        // Start frame
        buffer[..START_FRAME_SIZE].fill(0x00);

        // LED frames
        for (i, &RGB8 { r, g, b }) in colors.iter().enumerate().take(num_leds) {
            let offset = START_FRAME_SIZE + i * LED_FRAME_SIZE;
            // Each color is 7 bits with the highest bit set to 1
            buffer[offset] = (g >> 1) | 0x80;
            buffer[offset + 1] = (r >> 1) | 0x80;
            buffer[offset + 2] = (b >> 1) | 0x80;
        }

        // End frame
        let end_frame_offset = START_FRAME_SIZE + num_leds * LED_FRAME_SIZE;
        buffer[end_frame_offset..end_frame_offset + END_FRAME_SIZE].fill(0x00);

        // Write the data
        let total_size = START_FRAME_SIZE + num_leds * LED_FRAME_SIZE + END_FRAME_SIZE;
        self.spi.write(&buffer[..total_size]).await
    }
}
