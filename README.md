# Rust LED Driver Crate

This crate provides drivers for popular addressable LED strips, including **WS2812**, **APA102**, **LPD8806**, and **SK6812**. It offers a unified interface to control an arbitrary number of LEDs using the `embedded-hal-async` SPI bus.

## Features

*Multiple LED Protocols*: Supports WS2812, APA102, LPD8806, and SK6812.

*Async SPI Communication*: Utilizes the `embedded-hal-async` crate for LED data transmission.

*Customizable Color Ordering*: Supports dynamic color ordering (RGB, GRB).

*Buffer Management*: Automatically calculates the necessary buffer size for LED data and reset signals.

*Easy Integration*: Simple API to write RGB/RGBW data to the LED strip.
