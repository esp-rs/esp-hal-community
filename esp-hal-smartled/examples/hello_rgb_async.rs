//! RGB LED Demo - Async version.
//!
//! This example drives an SK68XX RGB LED, which is connected to a pin on the
//! official DevKits.
//!
//! It is the exact same as the `hello_rgb` example,
//! except it uses the async driver on top of embassy.
//!
//! Requires the `defmt` feature.

//% CHIPS: esp32 esp32c3 esp32c6 esp32h2 esp32s2 esp32s3

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{rmt::Rmt, time::Rate};
use esp_hal_smartled::{RmtSmartLeds, Ws2812Timing, buffer_size, color_order};
use smart_leds::RGB8;
use smart_leds::{
    SmartLedsWriteAsync, brightness, gamma,
    hsv::{Hsv, hsv2rgb},
};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let software_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

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

    // Increase LED count as needed.
    const LEDS: usize = 1;

    let mut led = {
        let rmt = Rmt::new(peripherals.RMT, freq)
            .expect("Failed to initialize RMT0")
            .into_async();
        // Configure color order and timing implementation as needed.
        RmtSmartLeds::<{ buffer_size::<RGB8>(LEDS) }, _, RGB8, color_order::Rgb, Ws2812Timing>::new(
            rmt.channel0,
            led_pin,
        )
        .unwrap()
    };

    let mut color = Hsv {
        hue: 0,
        sat: 255,
        val: 255,
    };
    let mut data;

    spawner.must_spawn(background_print());

    loop {
        // Iterate over the rainbow!
        for hue in 0..=255 {
            color.hue = hue;
            // Convert from the HSV color space (where we can easily transition from one
            // color to the other) to the RGB color space that we can then send to the LED
            data = [hsv2rgb(color); LEDS];
            // When sending to the LED, we do a gamma correction first (see smart_leds
            // documentation for details) and then limit the brightness to 10 out of 255 so
            // that the output it's not too bright.

            // This call already prepares the buffer.
            let fut = led.write(brightness(gamma(data.iter().cloned()), 10));
            // Put more led.write() calls (for other drivers) and other peripheral preparations here...

            // Dispatch all the LED writes at once.
            // (We simulate the second write instead with a delay.)
            let (_, res) = join(Timer::after_millis(20), fut).await;
            res.unwrap();
        }
    }
}

// Example task to demonstrate that while the LED operation is going on,
// something else can be done too with the power of async!
#[embassy_executor::task]
async fn background_print() {
    loop {
        defmt::info!("Hello from the background!");
        Timer::after_secs(2).await;
    }
}
