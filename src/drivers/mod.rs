pub mod apa102;
pub mod lpd8806;
pub mod sk6812;
pub mod ws2812;
pub trait LedDriver<Color> {
    type Error;
    async fn write(&mut self, colors: &[Color]) -> Result<(), Self::Error>;
}
