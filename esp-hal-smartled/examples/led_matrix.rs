//! 2D LED matrix demo.
//!
//! This example demonstrates the use of an LED strip arranged as a 2D matrix with `embedded-graphics`.
//! The LED strip consists of WS2812B RGB LEDs, is connected on GPIO 3 and is assumed to have a size of 16x16 (=256 LEDs).

//% CHIPS: esp32 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3

#![no_std]
#![no_main]

use embassy_time::Timer;
use embedded_graphics::{
    pixelcolor::*,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use esp_backtrace as _;
use esp_hal::{delay::Delay, rmt::Rmt, time::Rate};
use esp_hal_smartled::{
    RmtSmartLedsGraphics, Ws2812bTiming, color_order, graphics::buffer_size_2d,
};
use smart_leds::RGB8;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let led_pin = peripherals.GPIO3;

    // Configure RMT peripheral globally
    cfg_if::cfg_if! {
        if #[cfg(feature = "esp32h2")] {
            let freq = Rate::from_mhz(32);
        } else {
            let freq = Rate::from_mhz(80);
        }
    }

    // Use width = height for a square panel.
    const WIDTH: usize = 7;

    let mut display = {
        let rmt = Rmt::new(peripherals.RMT, freq).expect("Failed to initialize RMT0");
        RmtSmartLedsGraphics::<
            RGB8,                                     // Color type (8-bit RGB)
            color_order::Grb,                         // Color order (GRB)
            Ws2812bTiming,                            // LED timing type (WS2812B)
            { buffer_size_2d::<RGB8>(WIDTH, WIDTH) }, // Buffer size, automatically calculated
            WIDTH,                                    // Width
            WIDTH,                                    // Height
            false, // Use snaking arrangement. This is how self-made matrices are usually arranged.
        >::new_with_memsize(
            // To use 4 buffers we need to use TX channel 0.
            rmt.channel0,
            led_pin,
            // Use all 4 DMA buffers to make the panel as fast as possible;
            // this prevents any other RMT channels from being used.
            4,
        )
        .unwrap()
    };
    let delay = Delay::new();

    // Make a 3x3 rectangle.
    let mut rect = Rectangle::new(Point::new(1, 1), Size::new(3, 3));

    loop {
        // Draw the rectangle onto the display after clearing it.
        display.clear(Rgb888::BLACK).unwrap();
        rect.into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb888::CSS_DARK_RED)
                .build(),
        )
        .draw(&mut display)
        .unwrap();
        // Transmit pixel data.
        display.flush().unwrap();

        delay.delay_millis(1000);

        // Move rectangle right by 1 pixel, and reset to the left edge after it leaves the screen.
        rect = rect.translate(Point::new(1, 0));
        if rect.top_left.x >= WIDTH as i32 {
            rect.top_left.x = 0;
        }
    }
}
