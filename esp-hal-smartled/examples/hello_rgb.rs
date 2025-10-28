//! RGB LED Demo
//!
//! This example drives an SK68XX RGB LED, which is connected to a pin on the
//! official DevKits.
//!
//! The demo will leverage the [`smart_leds`](https://crates.io/crates/smart-leds)
//! crate functionality to circle through the HSV hue color space (with
//! saturation and value both at 255). Additionally, we apply a gamma correction
//! and limit the brightness to 10 (out of 255).
//!
//! The following wiring is assumed for ESP32:
//! - LED => GPIO33
//! The following wiring is assumed for ESP32C3:
//! <https://github.com/esp-rs/esp-rust-board/tree/master>
//! - LED => GPIO2
//! The following wiring is assumed for ESP32C6, ESP32H2:
//! <https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32c6/esp32-c6-devkitm-1/user_guide.html>
//! <https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32h2/esp32-h2-devkitm-1/user_guide.html>
//! - LED => GPIO8
//! The following wiring is assumed for ESP32S2:
//! <https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32s2/esp32-s2-devkitm-1/user_guide.html>
//! - LED => GPIO18
//! The following wiring is assumed for ESP32S3:
//! <https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32s3/esp32-s3-devkitm-1/user_guide.html>
//! - LED => GPIO48

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{delay::Delay, main, rmt::Rmt, time::Rate};
use esp_hal_smartled::{SmartLedsAdapter, smart_led_buffer};
use smart_leds::{
    RGB8, SmartLedsWrite, brightness, gamma,
    hsv::{Hsv, hsv2rgb},
};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Initialize the HAL Peripherals
    let p = esp_hal::init(esp_hal::Config::default());

    // Configure RMT (Remote Control Transceiver) peripheral globally
    // <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/peripherals/rmt.html>
    let rmt: Rmt<'_, esp_hal::Blocking> = {
        let frequency: Rate = {
            cfg_if::cfg_if! {
                if #[cfg(feature = "esp32h2")] {
                    Rate::from_mhz(32)
                } else {
                    Rate::from_mhz(80)
                }
            }
        };
        Rmt::new(p.RMT, frequency)
    }
    .expect("Failed to initialize RMT");

    // We use one of the RMT channels to instantiate a `SmartLedsAdapter` which can
    // be used directly with all `smart_led` implementations
    let rmt_channel = rmt.channel0;
    let rmt_buffer = smart_led_buffer!(1);

    // Each devkit uses a unique GPIO for the RGB LED, so in order to support
    // all chips we must unfortunately use `#[cfg]`s:
    let mut led = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "esp32")] {
                SmartLedsAdapter::new(rmt_channel, p.GPIO33, &mut rmt_buffer)
            } else if #[cfg(feature = "esp32c3")] {
                SmartLedsAdapter::new(rmt_channel, p.GPIO2, &mut rmt_buffer)
            } else if #[cfg(any(feature = "esp32c6", feature = "esp32h2"))] {
                SmartLedsAdapter::new(rmt_channel, p.GPIO8, &mut rmt_buffer)
            } else if #[cfg(feature = "esp32s2")] {
                SmartLedsAdapter::new(rmt_channel, p.GPIO18, &mut rmt_buffer)
            } else if #[cfg(feature = "esp32s3")] {
                SmartLedsAdapter::new(rmt_channel, p.GPIO48, &mut rmt_buffer)
            }
        }
    };

    let delay = Delay::new();

    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };
    let mut data: RGB8;
    let level = 10;

    loop {
        // Iterate over the rainbow!
        for hue in 0..=255 {
            color.hue = hue;
            // Convert from the HSV color space (where we can easily transition from one
            // color to the other) to the RGB color space that we can then send to the LED
            data = hsv2rgb(color);
            // When sending to the LED, we do a gamma correction first (see smart_leds docs
            // for details <https://docs.rs/smart-leds/latest/smart_leds/struct.Gamma.html>)
            // and then limit the brightness level to 10 out of 255 so that the output
            // is not too bright.
            led.write(brightness(gamma([data].into_iter()), level))
                .unwrap();
            delay.delay_millis(20);
        }
    }
}
