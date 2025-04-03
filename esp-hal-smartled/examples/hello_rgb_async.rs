//! Asynchronous RGB LED Demo
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
//! - LED => GPIO8
//! The following wiring is assumed for ESP32C6, ESP32H2:
//! - LED => GPIO8
//! The following wiring is assumed for ESP32S2:
//! - LED => GPIO18
//! The following wiring is assumed for ESP32S3:
//! - LED => GPIO48

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{rmt::Rmt, time::Rate, timer::timg::TimerGroup, Config};
use esp_hal_smartled::{buffer_size_async, SmartLedsAdapterAsync};
use smart_leds::{
    brightness, gamma,
    hsv::{hsv2rgb, Hsv},
    SmartLedsWriteAsync,
};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Each devkit uses a unique GPIO for the RGB LED, so in order to support
    // all chips we must unfortunately use `#[cfg]`s:
    cfg_if::cfg_if! {
        if #[cfg(feature = "esp32")] {
            let led_pin = peripherals.GPIO33;
        } else if #[cfg(feature = "esp32c3")] {
            let led_pin = peripherals.GPIO8;
        } else if #[cfg(any(feature = "esp32c6", feature = "esp32h2"))] {
            let led_pin = peripherals.GPIO8;
        } else if #[cfg(feature = "esp32s2")] {
            let led_pin = peripherals.GPIO18;
        } else if #[cfg(feature = "esp32s3")] {
            let led_pin = peripherals.GPIO48;
        }
    }

    // Configure RMT peripheral globally
    cfg_if::cfg_if! {
        if #[cfg(feature = "esp32h2")] {
            let freq = Rate::from_mhz(32);
        } else {
            let freq = Rate::from_mhz(80);
        }
    }

    let rmt = Rmt::new(peripherals.RMT, freq).unwrap().into_async();

    // We use one of the RMT channels to instantiate a `SmartLedsAdapter` which can
    // be used directly with all `smart_led` implementations
    let rmt_buffer: [u32; buffer_size_async(1)] = [0; buffer_size_async(1)];
    let mut led = SmartLedsAdapterAsync::new(rmt.channel0, led_pin, rmt_buffer);

    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };
    let mut data;

    loop {
        for hue in 0..=255 {
            color.hue = hue;
            // Convert from the HSV color space (where we can easily transition from one
            // color to the other) to the RGB color space that we can then send to the LED
            data = [hsv2rgb(color)];
            // When sending to the LED, we do a gamma correction first (see smart_leds
            // documentation for details) and then limit the brightness to 10 out of 255 so
            // that the output it's not too bright.
            led.write(brightness(gamma(data.iter().cloned()), 20)).await.unwrap();
            Timer::after(Duration::from_millis(10)).await;
        }
    }
}
