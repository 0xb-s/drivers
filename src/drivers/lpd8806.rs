use crate::drivers::LedDriver;
use embedded_hal_async::spi::SpiBus;
use heapless::Vec;
use smart_leds::RGB8;

pub struct Lpd8806<SPI: SpiBus<u8>, const N: usize> {
    spi: SPI,
    buffer: Vec<u8, N>,
}

const START_FRAME_SIZE: usize = 4;
const LED_FRAME_SIZE: usize = 3;

impl<SPI: SpiBus<u8>, const N: usize> Lpd8806<SPI, N> {
    /// Creates a new LPD8806 driver with the given SPI bus.
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            buffer: Vec::new(),
        }
    }

    /// Initializes the buffer with the required size for the given number of LEDs.
    fn initialize_buffer(&mut self, num_leds: usize) {
        let end_frame_size = (num_leds + 31) / 32;
        let total_size = START_FRAME_SIZE + num_leds * LED_FRAME_SIZE + end_frame_size;

        // Ensure buffer is cleared and resized based on the number of LEDs
        self.buffer.clear();
        self.buffer.resize(total_size, 0).ok(); // Resize the buffer, handle potential error silently
    }
}

impl<SPI: SpiBus<u8>, const N: usize> LedDriver<RGB8> for Lpd8806<SPI, N> {
    type Error = SPI::Error;

    async fn write(&mut self, colors: &[RGB8]) -> Result<(), Self::Error> {
        let num_leds = colors.len();

        // Initialize the buffer with the proper size based on the number of LEDs
        self.initialize_buffer(num_leds);

        // Start frame
        self.buffer[..START_FRAME_SIZE].fill(0x00);

        // LED frames
        for (i, &RGB8 { r, g, b }) in colors.iter().enumerate().take(num_leds) {
            let offset = START_FRAME_SIZE + i * LED_FRAME_SIZE;
            // Each color is 7 bits with the highest bit set to 1
            self.buffer[offset] = (g >> 1) | 0x80;
            self.buffer[offset + 1] = (r >> 1) | 0x80;
            self.buffer[offset + 2] = (b >> 1) | 0x80;
        }

        // End frame
        let end_frame_size = (num_leds + 31) / 32;
        let end_frame_offset = START_FRAME_SIZE + num_leds * LED_FRAME_SIZE;
        self.buffer[end_frame_offset..end_frame_offset + end_frame_size].fill(0x00);

        // Write the data
        let total_size = START_FRAME_SIZE + num_leds * LED_FRAME_SIZE + end_frame_size;
        self.spi.write(&self.buffer[..total_size]).await
    }
}
