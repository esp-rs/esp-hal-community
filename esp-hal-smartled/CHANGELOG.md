# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.17.0

### Changed

- Updated `esp-hal` to `v1.0.0` and updated related dependencies, including backwards-incompatible changes for updates to the `esp-hal::rmt` api ([rc0](https://docs.rs/esp-hal/1.0.0-rc.0/esp_hal/rmt/index.html), [rc1](https://docs.rs/esp-hal/1.0.0-rc.1/esp_hal/rmt/index.html)) like the new `PulseCode` type. See [esp-hal 1.0.0-rc.1 migration guide](https://github.com/esp-rs/esp-hal/blob/esp-hal-v1.0.0-rc.1/esp-hal/MIGRATING-1.0.0-rc.0.md) for details (#53, #57)

## 0.15.0

### Added

- New `SmartLedsAdapterAsync` which is an asynchronous, non-blocking version of the driver. (#6)
- Updated to use `esp-hal-beta.1`, see [migration guide](https://github.com/esp-rs/esp-hal/releases/tag/esp-hal-v1.0.0-beta.1) for details. (#31)

## 0.14.0

## 0.13.1

### Added

### Changed

### Fixed

### Removed

- Removed the `clocks` parameter from `SmartLedsAdapter::new` (#1999)

## 0.13.0 - 2024-08-29

## 0.12.0 - 2024-07-15

## 0.11.0 - 2024-06-04

## 0.10.0 - 2024-04-18

## 0.9.0 - 2024-03-08

## 0.8.0 - 2024-01-19

## 0.7.0 - 2023-12-12

## 0.6.0 - 2023-10-31

## 0.5.0 - 2023-09-05

## 0.4.0 - 2023-08-10

## 0.3.0 - 2023-07-04

## 0.2.0 - 2023-05-02

## 0.1.0 - 2023-03-27
