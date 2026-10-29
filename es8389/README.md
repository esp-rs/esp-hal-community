# es8389

[![Crates.io](https://img.shields.io/crates/v/es8389?labelColor=1C2C2E&color=C96329&logo=Rust&style=flat-square)](https://crates.io/crates/es8389)
[![docs.rs](https://img.shields.io/docsrs/es8389?labelColor=1C2C2E&color=C96329&logo=rust&style=flat-square)](https://docs.rs/es8389)
![Crates.io](https://img.shields.io/crates/l/es8389?labelColor=1C2C2E&style=flat-square)
[![Matrix](https://img.shields.io/matrix/esp-rs:matrix.org?label=join%20matrix&labelColor=1C2C2E&color=BEC5C9&logo=matrix&style=flat-square)](https://matrix.to/#/#esp-rs:matrix.org)

`no_std` async driver for the Everest Semiconductor ES8389 audio codec, built on
[`embedded-hal-async`](https://crates.io/crates/embedded-hal-async).

- Clock from MCLK or BCLK, at sample rates from 8 kHz to 192 kHz (see `Clock` for the supported
  clock frequencies)
- Philips I2S, left-justified, DSP-A and DSP-B formats with 16- to 32-bit words
- Microphone gain, DAC volume, ADC and DAC mute, standby and power-down
- Optional DAC reference in the right ADC channel for echo cancellation

The codec runs as an I2S slave with differential microphone inputs. The register sequences and
the coefficient table follow Espressif's
[`esp_codec_dev`](https://github.com/espressif/esp-adf/tree/master/components/esp_codec_dev)
ES8389 driver.

Tested on the ESP32-S31-Korvo-1, with BCLK as clock at 48 kHz and 16-bit Philips I2S, in
[korvo-voice-assistant](https://github.com/SergioGasquez/korvo-voice-assistant). The other
configurations follow the reference driver but are not tested on hardware.

## Example

```rust
use es8389::{Clock, Config, Es8389, Format, MicGain, WordLength};

let mut codec = Es8389::new(i2c, es8389::DEFAULT_ADDRESS);
// 16-bit samples in 32-bit slots: BCLK = 2 x 32 x 48 kHz.
codec
    .init(Config {
        clock: Clock::Bclk(3_072_000),
        sample_rate: 48_000,
        format: Format::I2s,
        word_length: WordLength::Bits16,
        dac_reference: false,
    })
    .await?;
codec.set_mic_gain(MicGain::Db30_5).await?;
```

The codec drops into analog standby without its clock, so start I2S before `init`.

## [Documentation]

[documentation]: https://docs.rs/es8389/

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

`src/coefficients.rs` and the register sequences derive from Espressif's `esp_codec_dev`,
Copyright 2025 Espressif Systems (Shanghai) CO LTD, licensed under the Apache License, Version 2.0.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
