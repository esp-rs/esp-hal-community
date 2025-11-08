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

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{Config, rmt::Rmt, time::Rate, timer::timg::TimerGroup};
use esp_hal_smartled::{SmartLedsAdapterAsync, buffer_size_async};
use smart_leds::{
    RGB8, SmartLedsWriteAsync, brightness, gamma,
    hsv::{Hsv, hsv2rgb},
};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize the HAL Peripherals
    let p = esp_hal::init(Config::default());
    #[cfg(target_arch = "riscv32")]
    {
        let timg0 = TimerGroup::new(p.TIMG0);
        let sw_interrupt =
            esp_hal::interrupt::software::SoftwareInterruptControl::new(p.SW_INTERRUPT);
        esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    }
    #[cfg(target_arch = "xtensa")]
    {
        let timg0 = TimerGroup::new(p.TIMG0);
        esp_rtos::start(timg0.timer0);
    }

    // Configure RMT (Remote Control Transceiver) peripheral globally
    // <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/peripherals/rmt.html>
    let rmt: Rmt<'_, esp_hal::Async> = {
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
    .expect("Failed to initialize RMT")
    .into_async();

    // We use one of the RMT channels to instantiate a `SmartLedsAdapterAsync` which can
    // be used directly with all `smart_led` implementations
    let rmt_channel = rmt.channel0;
    let mut rmt_buffer = [esp_hal::rmt::PulseCode::default(); buffer_size_async(1)];

    // Each devkit uses a unique GPIO for the RGB LED, so in order to support
    // all chips we must unfortunately use `#[cfg]`s:
    let mut led = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "esp32")] {
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO33, &mut rmt_buffer)
            } else if #[cfg(feature = "esp32c3")] {
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO2, &mut rmt_buffer)
            } else if #[cfg(any(feature = "esp32c6", feature = "esp32h2"))] {
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO8, &mut rmt_buffer)
            } else if #[cfg(feature = "esp32s2")] {
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO18, &mut rmt_buffer)
            } else if #[cfg(feature = "esp32s3")] {
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO48, &mut rmt_buffer)
            }
        }
    };

    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };
    let mut data: RGB8;
    let level = 10;

    loop {
        for hue in 0..=255 {
            color.hue = hue;
            // Convert from the HSV color space (where we can easily transition from one
            // color to the other) to the RGB color space that we can then send to the LED
            data = hsv2rgb(color);
            // When sending to the LED, we do a gamma correction first (see smart_leds
            // documentation for details) and then limit the brightness to 10 out of 255 so
            // that the output is not too bright.
            led.write(brightness(gamma([data].into_iter()), level))
                .await
                .unwrap();
            Timer::after(Duration::from_millis(10)).await;
        }
    }
}
