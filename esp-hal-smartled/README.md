# esp-hal-smartled

[![Crates.io](https://img.shields.io/crates/v/esp-hal-smartled2?labelColor=1C2C2E&color=C96329&logo=Rust&style=flat-square)](https://crates.io/crates/esp-hal-smartled2)
[![docs.rs](https://img.shields.io/docsrs/esp-hal-smartled2?labelColor=1C2C2E&color=C96329&logo=rust&style=flat-square)](https://docs.rs/esp-hal-smartled2)
![MSRV](https://img.shields.io/badge/MSRV-1.90-blue?labelColor=1C2C2E&style=flat-square)
![Crates.io](https://img.shields.io/crates/l/esp-hal-smartled2?labelColor=1C2C2E&style=flat-square)

> **NOTE:** This is an enhanced up-to-date fork of [esp-hal-smartled](https://crates.io/crates/esp-hal-smartled), which seems abandoned at the time of writing. Users of `esp-hal-smartled` can migrate to `esp-hal-smartled2` with minimal effort. If possible, this crate’s features will be merged into `esp-hal-smartled`.

Allows for the use of an RMT output channel on the ESP32 family to easily drive smart RGB LEDs. This is a driver for the [smart-leds](https://crates.io/crates/smart-leds) framework and allows using the utility functions from this crate as well as higher-level libraries based on smart-leds.

Different from [ws2812-esp32-rmt-driver](https://crates.io/crates/ws2812-esp32-rmt-driver), which is based on the unofficial `esp-idf` SDK, this crate is based on the official no-std [esp-hal](https://github.com/esp-rs/esp-hal). The RMT peripheral approach is common across many smart LED drivers on ESP32, like Arduino FastLED.

## Features

- **Configurability**: Use any color order and timing specification you want, either from the library itself, or your own. This makes `esp-hal-smartled2` compatible with many types of LEDs. Since both of these are determined at compile-time, the driver is always well-optimized for your specific LED type. (Currently, only RGB LEDs are supported, and RGBW/RGBWW/CC support might be added in the future.)
- **Async support**: The async write trait of smart-leds is supported, allowing you to use the driver without waiting for the LED write to complete.
- **No Allocation**: The driver uses only static buffers based on the maximum number of LEDs to drive, so you can use it without an allocator.

## [Documentation]

[documentation]: https://docs.rs/esp-hal-smartled2/

Also have a look at the [examples](https://github.com/kleinesfilmroellchen/esp-hal-smartled/blob/main/examples).

## Compatibility

This crate is guaranteed to compile on whatever Rust version esp-hal requires, which is the latest stable version at the time of release. It _might_ compile with older versions but that may change in any new patch release.

This crate uses the unstable RMT peripheral from esp-hal. Therefore, it is compatible with _exactly_ esp-hal 1.0.0, as the peripheral API might change in any future release and is likely to go through significant changes before stabilization. In order to use this crate, you have to enable the `unstable` feature on esp-hal.

### Migration

- `0.26`
  - `SmartLedsAdapter::new` now returns `Result<SmartLedsAdapter, RmtError>`, so that you can handle configuration errors if desired.
  - `SmartLedsAdapter::new_with_memsize` was added to specify a larger RMT memory size when desired.
- `0.25`
  - WS2811 timings have been changed to use fast timings instead of slow timings; slow timings are still available through `Ws2811LowSpeedTiming`. If you experience issues with WS2811, switch to the low-speed timing.
- `0.24`
  - `SmartLedsWriteAsync` is now implemented for async RMT channels. Refer to the [async example](https://github.com/kleinesfilmroellchen/esp-hal-smartled/blob/main/examples/hello_rgb_async.rs) for more information.
- `0.23` (from `esp-hal-smartled`):
  - `SmartLedAdapter` now takes type parameters describing the timing and LED buffer size.
  - It is no longer needed to allocate a separate buffer and pass it into `new`. The `smartLedBuffer!` macro has been removed, and the `buffer_size` function can be used instead to calculate the correct buffer size.
  - Have a look at the documentation or the [example](https://github.com/kleinesfilmroellchen/esp-hal-smartled/blob/main/examples/hello_rgb.rs) for details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
