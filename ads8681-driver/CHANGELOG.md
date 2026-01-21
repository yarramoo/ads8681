# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Major refactoring for modularity and clarity**
  - Split codebase into focused modules: `error`, `register`, `command`, `range`, and `tests`
  - Improved code organization with 6 well-defined modules
  - Enhanced documentation with 100% public API coverage
  - Added 13 runnable doc examples
  - Expanded test suite from 6 to 18 unit tests
  - Added `voltage_to_raw()` inverse conversion function
  - Implemented utility methods on enums (`is_bipolar()`, `is_unipolar()`, `name()`, etc.)
  - Added helper functions for command encoding/decoding
  - Improved error handling with From trait implementation
  - Added `#![deny(missing_docs)]` and `#![deny(unsafe_code)]` attributes

### Added
- `voltage_to_raw()` function for bidirectional voltage conversion
- `InputRange::from_register_value()` for safe range parsing
- Command helper functions: `encode_command()`, `decode_response()`, `extract_adc_data()`, `extract_register_data()`
- Register utility methods: `address()`, `is_read_only()`, `name()`
- Range utility methods: `value()`, `is_bipolar()`, `is_unipolar()`, `vref_multiplier()`, `name()`
- Command introspection methods: `opcode()`, `is_write()`, `is_read()`, `name()`
- Optional defmt support in error module
- CODE_REVIEW.md documenting refactoring improvements
- 12 additional unit tests for ranges and edge cases

### Fixed
- Corrected voltage_to_raw() doctest example

## [0.1.0] - 2026-01-21

### Added
- Initial release of ADS8681 driver
- Support for all programmable input ranges (bipolar and unipolar)
- Read 16-bit ADC conversions
- Device initialization and ID verification
- Register read/write operations
- Alarm threshold configuration
- `raw_to_voltage()` conversion helper function
- Comprehensive test coverage with mock SPI
- Mock verification example for hardware-free testing
- Full API documentation
- Support for embedded-hal 1.0 traits
- no_std compatibility

### Features
- 9 input range configurations
- SPI communication at up to 1 MHz
- Device reset functionality
- Error handling with custom Error type
- Platform-agnostic design using embedded-hal traits
