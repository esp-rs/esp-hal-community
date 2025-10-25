# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.3.0

### Changed

- Updated `esp-hal` to `v1.0.0-rc.1` and updated related dependencies, including breaking changes for `esp_hal::gpio::DriveMode`. See the [esp-hal 1.0.0-rc.0 migration guide](https://github.com/esp-rs/esp-hal/blob/main/esp-hal/MIGRATING-1.0.0-rc.0.md) for details (#53)
- Changed the pin in the example to GPIO4 (#53)

## 0.2.0

### Added

- Added `Buzzer::play_tones_from_slice(&self, sequence: &[u32], timings: &[u32])` to allow tone playback using slices (#39)

### Changed
- **Breaking Change:** `Buzzer::mute()` is now infallible (#38)
- **Breaking Change:** `Buzzer::play_song()` now takes a `&[ToneValue]` slice instead of a fixed-size `[ToneValue; N]` array (#39)

### Fixed
- Upgrade esp-hal to 1.0.0-beta.1 (#31)

### Removed
- **Breaking Change:** Generic for Buzzer has been removed in favour of AnyPin (#19)

## 0.1.0 - Initial release
