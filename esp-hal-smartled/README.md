# esp-hal-smartled

[![Crates.io](https://img.shields.io/crates/v/esp-hal-smartled2?labelColor=1C2C2E&color=C96329&logo=Rust&style=flat-square)](https://crates.io/crates/esp-hal-smartled2)
[![docs.rs](https://img.shields.io/docsrs/esp-hal-smartled2?labelColor=1C2C2E&color=C96329&logo=rust&style=flat-square)](https://docs.rs/esp-hal-smartled2)
![MSRV](https://img.shields.io/badge/MSRV-1.90-blue?labelColor=1C2C2E&style=flat-square)
![Crates.io](https://img.shields.io/crates/l/esp-hal-smartled2?labelColor=1C2C2E&style=flat-square)

> **NOTE:** This is an enhanced up-to-date fork of [esp-hal-smartled](https://crates.io/crates/esp-hal-smartled), which seems abandoned at the time of writing. Users of `esp-hal-smartled` can migrate to `esp-hal-smartled2` with minimal effort. If possible, this crate’s features will be merged into `esp-hal-smartled`.

Allows for the use of an RMT output channel on the ESP32 family to easily drive smart RGB LEDs. This is a driver for the [smart-leds](https://crates.io/crates/smart-leds) framework and allows using the utility functions from this crate as well as higher-level libraries based on smart-leds.

Different from [ws2812-esp32-rmt-driver](https://crates.io/crates/ws2812-esp32-rmt-driver), which is based on the unofficial `esp-idf` SDK, this crate is based on the official no-std [esp-hal](https://github.com/esp-rs/esp-hal).

## [Documentation]

[documentation]: https://docs.rs/esp-hal-smartled2/

## Compatibility

This crate is guaranteed to compile on whatever Rust version esp-hal requires, which is the latest stable version at the time of release. It _might_ compile with older versions but that may change in any new patch release.

This crate uses the unstable RMT peripheral from esp-hal. Therefore, it is compatible with *exactly* esp-hal 1.0.0, as the peripheral API might change in any future release and is likely to go through significant changes before stabilization. In order to use this crate, you have to enable the `unstable` feature on esp-hal.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
