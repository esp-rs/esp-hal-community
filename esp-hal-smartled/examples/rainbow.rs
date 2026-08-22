//! Rainbow Demo
//!
//! This example drives up to 60 RGB LEDs to display a running rainbow.
//!
//! The following wiring is assumed for ESP32:
//! - LED => GPIO33
//!
//! The following wiring is assumed for ESP32C3:
//! - LED => GPIO8
//!
//! The following wiring is assumed for ESP32C6, ESP32H2:
//! - LED => GPIO8
//!
//! The following wiring is assumed for ESP32S2:
//! - LED => GPIO18
//!
//! The following wiring is assumed for ESP32S3:
//! - LED => GPIO48
//!
//! You might need to adjust the color order and timing types during the [`RmtSmartLeds`] initialization.

//% CHIPS: esp32 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3

#![no_std]
#![no_main]

use core::cfg_select;
use esp_backtrace as _;
use esp_hal::{delay::Delay, rmt::Rmt, time::Rate};
use esp_hal_smartled::{RmtSmartLeds, buffer_size, color_order};
use smart_leds::{
    RGB8, SmartLedsWrite, brightness, gamma,
    hsv::{Hsv, hsv2rgb},
};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Each devkit uses a unique GPIO for the RGB LED.
    cfg_select! {
        feature = "esp32" => {
            let led_pin = peripherals.GPIO33;
        }
        feature = "esp32c3" => {
            let led_pin = peripherals.GPIO8;
        }
        any(feature = "esp32c6", feature = "esp32h2") => {
            let led_pin = peripherals.GPIO8;
        }
        feature = "esp32s2" => {
            let led_pin = peripherals.GPIO18;
        }
        feature = "esp32s3" => {
            let led_pin = peripherals.GPIO48;
        }
    }

    // Configure RMT peripheral globally
    cfg_select! {
        feature = "esp32h2" => {
            let freq = Rate::from_mhz(32);
        }
        _ => {
            let freq = Rate::from_mhz(80);
        }
    }

    const COUNT: usize = 60;
    type LedColor = RGB8;
    let mut led = {
        let rmt = Rmt::new(peripherals.RMT, freq).expect("Failed to initialize RMT0");
        // Configure color order and timing implementation as needed.
        RmtSmartLeds::<{ buffer_size::<LedColor>(COUNT) }, _, LedColor, color_order::Grb>::new_with_memsize(
            esp_hal_smartled::WS2812_TIMING.with_reset_us(305), // custom reset time
            rmt.channel0,
            led_pin,
            2,
            freq,
        )
        .unwrap()
    };
    let delay = Delay::new();

    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };
    let mut data = [hsv2rgb(color); COUNT];

    let mut start_hue = 0;
    loop {
        start_hue += 1;
        // Iterate over the rainbow!
        for hue in 0..(COUNT as u8) {
            color.hue = hue.wrapping_add(start_hue);
            // Convert from the HSV color space (where we can easily transition from one
            // color to the other) to the RGB color space that we can then send to the LED
            data[hue as usize] = hsv2rgb(color);
        }
        // When sending to the LED, we do a gamma correction first (see smart_leds
        // documentation for details) and then limit the brightness to 10 out of 255 so
        // that the output it's not too bright.
        led.write(brightness(gamma(data.iter().cloned()), 10))
            .unwrap();
        delay.delay_millis(5);
    }
}
