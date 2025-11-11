# esp-hal-smartled

[![Crates.io](https://img.shields.io/crates/v/esp-hal-smartled2?labelColor=1C2C2E&color=C96329&logo=Rust&style=flat-square)](https://crates.io/crates/esp-hal-smartled2)
[![docs.rs](https://img.shields.io/docsrs/esp-hal-smartled2?labelColor=1C2C2E&color=C96329&logo=rust&style=flat-square)](https://docs.rs/esp-hal-smartled2)
![MSRV](https://img.shields.io/badge/MSRV-1.90-blue?labelColor=1C2C2E&style=flat-square)
![Crates.io](https://img.shields.io/crates/l/esp-hal-smartled2?labelColor=1C2C2E&style=flat-square)

> **NOTE:** This is an enhanced up-to-date fork of [esp-hal-smartled](https://crates.io/crates/esp-hal-smartled), which seems abandoned. Users of `esp-hal-smartled` can migrate to `esp-hal-smartled2` with some effort.

Allows for the use of an RMT output channel on the ESP32 family to easily drive smart RGB LEDs. This is a driver for the [smart-leds](https://crates.io/crates/smart-leds) framework and allows using the utility functions from this crate as well as higher-level libraries based on smart-leds.

Different from [ws2812-esp32-rmt-driver](https://crates.io/crates/ws2812-esp32-rmt-driver), which is based on the unofficial `esp-idf` SDK, this crate is based on the official no-std [esp-hal](https://github.com/esp-rs/esp-hal). The RMT peripheral approach is common across many smart LED drivers on ESP32, like Arduino FastLED.

## Features

- **Configurability**: `esp-hal-smartled2` works with:
  - any (plausible) `smart-led` color type, including RGB, RGBW, RGBCCT, CCT, in 8, 16, 32 or 64 bits.
  - any color order; all six RGB orders are predefined.
  - any timing specification (within range of the RMT peripheral); common LED types have predefined timings, but custom ones are supported.

  This makes `esp-hal-smartled2` compatible with many configurations of LEDs, and almost the entire `smart-leds` featureset. Since all of these are determined at compile-time, the driver is always well-optimized for your specific LED type.

- **Async support**: The `SmartLedsWriteAsync` trait of smart-leds is supported, allowing you to use the driver without waiting for the LED write to complete.
- **No Allocation**: The driver uses only static buffers based on the maximum number of LEDs to drive, so you can use it without an allocator.

## [Documentation]

[documentation]: https://docs.rs/esp-hal-smartled2/

Also have a look at the [examples](https://github.com/kleinesfilmroellchen/esp-hal-smartled/blob/main/examples).

## Compatibility

This crate is guaranteed to compile on whatever Rust version esp-hal requires, which is the latest stable version at the time of release. It _might_ compile with older versions but that may change in any new patch release.

This crate uses the unstable RMT peripheral from esp-hal. Therefore, it is compatible with _exactly_ esp-hal 1.0.0, as the peripheral API might change in any future release and is likely to go through significant changes before stabilization. In order to use this crate, you have to enable the `unstable` feature on esp-hal.

### Migration

- `0.28`
  - Renamed the driver from `SmartLedsAdapter` to the more descriptive `RmtSmartLeds` to emphasize the use of the RMT peripheral.
- `0.27`
  - `RmtSmartLeds` and `buffer_size` now take a `Color` type parameter (after the transmit mode). This allows you to use color types other than RGB8, including ones with larger bit widths. `Rgb8RmtSmartLeds` is a convenience alias for common RGB8-based LEDs, and works like `RmtSmartLeds` did before.
  - `ColorOrder` is now a generic trait over a color type, since some color orders work with multiple color types (e.g. all RGB orders work with all RGB color types, regardless of bit width). The existing order types work as before.
  - `Channel` has been removed, as the channel count can now be larger or smaller depending on the color type. Since this is not really enforceable at compile time, care must be taken when passing channel numbers to `ColorOrder::get_channel_data()`.
  - Async implementation cooperates better with parallelization. When you call the async `write` function now, it immediately prepares the driver for sending. The actual send is still only dispatched once you `await` it. The rewritten async example shows how this can be used in practice to prepare the buffer in advance, and dispatch multiple LED writes simultaneously using `join`.
- `0.26`
  - `RmtSmartLeds::new` now returns `Result<RmtSmartLeds, RmtError>`, so that you can handle configuration errors if desired.
  - `RmtSmartLeds::new_with_memsize` was added to specify a larger RMT memory size when desired.
- `0.25`
  - WS2811 timings have been changed to use fast timings instead of slow timings; slow timings are still available through `Ws2811LowSpeedTiming`. If you experience issues with WS2811, switch to the low-speed timing.
- `0.24`
  - `SmartLedsWriteAsync` is now implemented for async RMT channels. Refer to the [async example](https://github.com/kleinesfilmroellchen/esp-hal-smartled/blob/main/examples/hello_rgb_async.rs) for more information.
- `0.23` (from `esp-hal-smartled`):
  - `SmartLedAdapter` now takes type parameters describing the timing and LED buffer size.
  - It is no longer needed to allocate a separate buffer and pass it into `new`. The `smartLedBuffer!` macro has been removed, and the `buffer_size` function can be used instead to calculate the correct buffer size.
  - Have a look at the documentation or the [example](https://github.com/kleinesfilmroellchen/esp-hal-smartled/blob/main/examples/hello_rgb.rs) for details.

### Future features

- Release a version 1 once the RMT peripheral is stable.
- LED data streaming via continuous RMT transmission. This would massively improve the throughput of the driver, especially for large LED counts. It needs support on the RMT peripheral side in `esp-hal`, where such work is explicitly planned.

If you really need one of them, please tell me about it!

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
