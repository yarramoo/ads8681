# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-21

### Added
- Initial release of ADS8681 driver
- Support for all programmable input ranges (bipolar and unipolar)
- Read 16-bit ADC conversions
- Device initialization and ID verification
- Register read/write operations
- Alarm threshold configuration
- Voltage conversion helper function
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
