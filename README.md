# esp-hal-community

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/esp-rs/esp-hal-community/ci.yml?labelColor=1C2C2E&label=CI&logo=github&style=flat-square)
![MIT/Apache-2.0 licensed](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?labelColor=1C2C2E&style=flat-square)
[![Matrix](https://img.shields.io/matrix/esp-rs:matrix.org?labelColor=1C2C2E&label=join%20matrix&color=BEC5C9&logo=matrix&style=flat-square)](https://matrix.to/#/#esp-rs:matrix.org)

A collection of crates for use alongside [esp-hal], but which are maintained by the community.

[esp-hal]: https://github.com/esp-rs/esp-hal/

## Examples

The following command can be used to run the smart LED example on a ESP32C6 target.
```bash
cargo +stable run --example hello_rgb --features "esp32c6,esp-hal/unstable" --target=riscv32imac-unknown-none-elf --release
```

This repository also provides a [`justfile`](https://github.com/casey/just) which provides
`just run-esp32c6 hello_rgb` as a shorthand for the command above. There are also shorthands for
running other examples and running on other chips. You can use `just --list` to list all provided
shorthands.

There are two sample toolchain files available for simplifying development:
`rust-toolchain-risc-v.toml` and `rust-toolchain-xtensa.toml`. Depending on which target you develop
for, you can use `cp rust-toolchain-risc-v.toml rust-toolchain.toml` or
`cp rust-toolchain-xtensa.toml rust-toolchain.toml` to set the correct Rust toolchain.

## Contributing a Crate

If you have a crate which depends on `esp-hal` and provides some additional functionality, we encourage you to contribute it to this repository!

When opening a pull request to add a new crate, we ask that you please ensure the following criteria are met:

- The new crate follows the `esp-hal-*` naming convention
- The new crate contains `CHANGELOG.md` and `README.md` files, following the formatting of other crates in this repository
- The new crate is added to the CI workflow
- The new crate has been added to `CODEOWNERS` along with your username
  - If you are unable or unwilling to take ownership of this crate for whatever reason, please state such in your pull request and we can try to find an owner for it

Upon approval of your pull request, you will be granted Maintainer privileges on the repository and be added as an owner on [crates.io], assuming you are commiting to be code owner for the added crate.

[crates.io]: https://crates.io

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution notice

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
