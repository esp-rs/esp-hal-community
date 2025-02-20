# esp-hal-buzzer

[![Crates.io](https://img.shields.io/crates/v/esp-hal-buzzer?labelColor=1C2C2E&color=C96329&logo=Rust&style=flat-square)](https://crates.io/crates/esp-hal-buzzer)
[![docs.rs](https://img.shields.io/docsrs/esp-hal-buzzer?labelColor=1C2C2E&color=C96329&logo=rust&style=flat-square)](https://docs.rs/esp-hal-buzzer)
![MSRV](https://img.shields.io/badge/MSRV-1.76-blue?labelColor=1C2C2E&style=flat-square)
![Crates.io](https://img.shields.io/crates/l/esp-hal-buzzer?labelColor=1C2C2E&style=flat-square)
[![Matrix](https://img.shields.io/matrix/esp-rs:matrix.org?label=join%20matrix&labelColor=1C2C2E&color=BEC5C9&logo=matrix&style=flat-square)](https://matrix.to/#/#esp-rs:matrix.org)

Provides a driver to easily interact with piezo-electric buzzers for `esp-hal`. The crate uses the underlying Ledc driver and provides a user-friendly API.

A few songs are included in the [songs](./src/songs.rs) module. Contributions are welcome.

## [Documentation]

[documentation]: https://docs.rs/esp-hal-buzzer/

## Minimum Supported Rust Version (MSRV)

This crate is guaranteed to compile on stable Rust 1.84 and up. It _might_
compile with older versions but that may change in any new patch release.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
