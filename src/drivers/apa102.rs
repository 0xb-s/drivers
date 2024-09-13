use embedded_hal_async::spi::SpiBus;
use smart_leds::RGB8;

use crate::drivers::LedDriver;

pub struct Apa102<SPI: SpiBus<u8>> {
    spi: SPI,
    buffer: [u8; MAX_BUFFER_SIZE],
}

const MAX_LEDS: usize = 60; // Adjust as needed
const START_FRAME_SIZE: usize = 4;
const LED_FRAME_SIZE: usize = 4;
const END_FRAME_SIZE: usize = (MAX_LEDS + 15) / 16;
const MAX_BUFFER_SIZE: usize = START_FRAME_SIZE + (MAX_LEDS * LED_FRAME_SIZE) + END_FRAME_SIZE;

impl<SPI: SpiBus<u8>> Apa102<SPI> {
    /// Creates a new APA102 driver with the given SPI bus.
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            buffer: [0; MAX_BUFFER_SIZE],
        }
    }
}

impl<SPI: SpiBus<u8>> LedDriver<RGB8> for Apa102<SPI> {
    type Error = SPI::Error;

    async fn write(&mut self, colors: &[RGB8]) -> Result<(), Self::Error> {
        let num_leds = core::cmp::min(colors.len(), MAX_LEDS);
        let end_frame_size = (num_leds + 15) / 16;

        // Prepare the buffer.
        let buffer = &mut self.buffer[..];

        // Start frame
        buffer[..START_FRAME_SIZE].fill(0x00);

        // LED frames
        for (i, &RGB8 { r, g, b }) in colors.iter().enumerate().take(num_leds) {
            let offset = START_FRAME_SIZE + i * LED_FRAME_SIZE;
            // Data frame: 0xE0 | brightness (set to max 0x1F), then B, G, R.
            buffer[offset] = 0xE0 | 0x1F; // Global brightness set to max (0x1F)
            buffer[offset + 1] = b;
            buffer[offset + 2] = g;
            buffer[offset + 3] = r;
        }

        // End frame
        let end_frame_offset = START_FRAME_SIZE + num_leds * LED_FRAME_SIZE;
        buffer[end_frame_offset..end_frame_offset + end_frame_size].fill(0x00);

        // Write the data
        let total_size = START_FRAME_SIZE + num_leds * LED_FRAME_SIZE + end_frame_size;
        self.spi.write(&buffer[..total_size]).await
    }
}
