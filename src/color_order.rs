use smart_leds::RGBW;
/// The order of the colors
#[derive(Clone, Copy)]
pub enum ColorOrder {
    RGB,
    GRB,
}

pub type RGBW8 = RGBW<u8>;
