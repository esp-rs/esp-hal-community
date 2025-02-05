# esp-hal-community

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/esp-rs/esp-hal-community/ci.yml?labelColor=1C2C2E&label=CI&logo=github&style=flat-square)
![MIT/Apache-2.0 licensed](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?labelColor=1C2C2E&style=flat-square)
[![Matrix](https://img.shields.io/matrix/esp-rs:matrix.org?labelColor=1C2C2E&label=join%20matrix&color=BEC5C9&logo=matrix&style=flat-square)](https://matrix.to/#/#esp-rs:matrix.org)

A collection of crates for use alongside [esp-hal], but which are maintained by the community.

[esp-hal]: https://github.com/esp-rs/esp-hal/

## Examples

To run the examples for either crate, either open the project at the sub-crate level or change directory:

```bash
# cd into crate directory for smartled or buzzer
cd esp-hal-smartled
# cargo <chip alias> --example <example name>
cargo esp32c3 --example hello_rgb # or other chip
```

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
