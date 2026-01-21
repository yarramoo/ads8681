# ADS8681 Driver

[![Crates.io](https://img.shields.io/crates/v/ads8681-driver.svg)](https://crates.io/crates/ads8681-driver)
[![Documentation](https://docs.rs/ads8681-driver/badge.svg)](https://docs.rs/ads8681-driver)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](README.md#license)

Platform-agnostic Rust driver for the Texas Instruments ADS8681 16-bit, 1 MSPS, Single-Supply SAR ADC with Programmable Bipolar Input Ranges.

This driver uses the `embedded-hal` traits, making it portable across different embedded platforms.

## Features

- Read 16-bit ADC conversions
- Configure programmable input ranges (±12.288V to ±2.56V bipolar, 0-12.288V to 0-5.12V unipolar)
- Read and write device registers
- Verify device communication via device ID check
- Configure alarm thresholds
- Convert raw ADC values to voltages
- `no_std` compatible
- Comprehensive test coverage with mock SPI

## Hardware Overview

The ADS8681 is a 16-bit SAR ADC with:
- 1 MSPS sampling rate
- SPI interface (up to 1 MHz)
- Programmable input ranges via software
- Built-in 4.096V reference
- Overvoltage protection up to ±20V
- Single 5V supply operation

### Input Ranges

The ADS8681 supports the following input ranges (with 4.096V reference):

**Bipolar Ranges:**
- ±12.288V (±3 × Vref)
- ±10.24V (±2.5 × Vref)
- ±6.144V (±1.5 × Vref)
- ±5.12V (±1.25 × Vref)
- ±2.56V (±0.625 × Vref)

**Unipolar Ranges:**
- 0 to 12.288V (3 × Vref)
- 0 to 10.24V (2.5 × Vref)
- 0 to 6.144V (1.5 × Vref)
- 0 to 5.12V (1.25 × Vref)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ads8681-driver = "0.1"
embedded-hal = "1.0"
```

## Usage

### Basic Example

```rust
use ads8681_driver::{ADS8681, InputRange, raw_to_voltage};

// Create driver instance (platform-specific SPI and CS pin setup)
let mut adc = ADS8681::new(spi, cs_pin);

// Initialize and verify device
adc.init().expect("Failed to initialize ADS8681");

// Configure input range to ±10.24V
adc.set_input_range(InputRange::BipolarTwoPointFiveVref)
    .expect("Failed to set range");

// Read ADC value
let raw_value = adc.read_adc().expect("Failed to read ADC");

// Convert to voltage (4.096V reference)
let voltage = raw_to_voltage(raw_value, InputRange::BipolarTwoPointFiveVref, 4.096);
println!("Measured voltage: {:.3}V", voltage);
```

### Platform-Specific Examples

#### STM32 with embedded-hal 1.0

```rust
use stm32f4xx_hal::{pac, prelude::*, spi::Spi};
use ads8681_driver::{ADS8681, InputRange};

let dp = pac::Peripherals::take().unwrap();
let rcc = dp.RCC.constrain();
let clocks = rcc.cfgr.freeze();

let gpioa = dp.GPIOA.split();

// Configure SPI
let sck = gpioa.pa5.into_alternate();
let miso = gpioa.pa6.into_alternate();
let mosi = gpioa.pa7.into_alternate();
let cs = gpioa.pa4.into_push_pull_output();

let spi = Spi::new(
    dp.SPI1,
    (sck, miso, mosi),
    embedded_hal::spi::MODE_0,
    1.MHz(),
    &clocks,
);

let mut adc = ADS8681::new(spi, cs);
adc.init().unwrap();
```

#### Raspberry Pi with linux-embedded-hal

```rust
use linux_embedded_hal::{SpidevBus, Pin};
use ads8681_driver::{ADS8681, InputRange};

let spi = SpidevBus::open("/dev/spidev0.0").unwrap();
let cs = Pin::new(25); // GPIO 25

let mut adc = ADS8681::new(spi, cs);
adc.init().unwrap();
```

### Reading Multiple Samples

```rust
// Continuous reading
for _ in 0..100 {
    let value = adc.read_adc().unwrap();
    let voltage = raw_to_voltage(value, InputRange::BipolarThreeVref, 4.096);
    println!("{:.3}V", voltage);

    // Add delay between readings as needed
    delay.delay_ms(10);
}
```

### Alarm Configuration

```rust
// Set alarm thresholds
adc.set_alarm_high_threshold(0x6000).unwrap();
adc.set_alarm_low_threshold(0x2000).unwrap();

// Read back thresholds
let high = adc.get_alarm_high_threshold().unwrap();
let low = adc.get_alarm_low_threshold().unwrap();
```

## Verification

The driver includes comprehensive verification examples that can run without hardware:

```bash
# Run all tests
cargo test

# Run mock verification example
cargo run --example mock_verification
```

The mock verification example demonstrates:
- Device initialization and ID verification
- Input range configuration
- ADC value reading
- Alarm threshold configuration
- Voltage conversion calculations

## API Documentation

### Main Types

- `ADS8681<SPI, CS>` - Main driver struct
- `InputRange` - Enum of available input ranges
- `Register` - Device register addresses
- `Command` - SPI command opcodes
- `Error<E>` - Error types

### Key Methods

- `new(spi, cs)` - Create driver instance
- `init()` - Initialize and verify device
- `read_adc()` - Read 16-bit ADC value
- `set_input_range(range)` - Configure input range
- `get_input_range()` - Read current input range
- `read_register(reg)` - Read device register
- `write_register(reg, value)` - Write device register
- `reset()` - Reset the device
- `release()` - Consume driver and return SPI/CS

### Helper Functions

- `raw_to_voltage(raw, range, vref)` - Convert raw ADC value to voltage

## SPI Protocol

The ADS8681 uses a 4-byte SPI command structure:

```
Byte 0: [Command (7 bits)] [Address bit 8]
Byte 1: Address bits [7:0]
Byte 2: Data MSB
Byte 3: Data LSB
```

Response format:
- ADC data is returned in the upper 16 bits of the 32-bit response
- Register read requires two transactions: READ command, then NOP to retrieve data

## Register Map

| Register | Address | Description |
|----------|---------|-------------|
| DEVICE_ID | 0x00 | Device identification (0x0681) |
| RST_PWRCTL | 0x04 | Reset and power control |
| SDI_CTL | 0x08 | SDI control |
| SDO_CTL | 0x0C | SDO control |
| DATAOUT_CTL | 0x10 | Data output control |
| RANGE_SEL | 0x14 | Input range selection |
| ALARM | 0x20 | Alarm configuration |
| ALARM_H_TH | 0x24 | Alarm high threshold |
| ALARM_L_TH | 0x28 | Alarm low threshold |

## Testing

The driver includes comprehensive tests using mock SPI:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_voltage_conversion_bipolar
```

## Examples

- `mock_verification.rs` - Comprehensive verification without hardware
- `basic_usage.rs` - Template for real hardware integration

## Troubleshooting

### Device ID Mismatch
If `init()` returns `InvalidDeviceId`, check:
- SPI wiring (SCLK, MISO, MOSI, CS)
- SPI mode (should be MODE_0: CPOL=0, CPHA=0)
- SPI clock speed (max 1 MHz for ADS8681)
- Power supply (5V on AVDD)

### Incorrect Readings
- Verify correct input range is configured
- Check reference voltage (should be 4.096V typically)
- Ensure input is within selected range
- Allow settling time after range changes

### Communication Errors
- Reduce SPI clock speed
- Check CS pin polarity (active low)
- Verify embedded-hal implementation

## References

- [ADS8681 Datasheet](https://www.ti.com/lit/ds/symlink/ads8681.pdf)
- [ADS8681 Product Page](https://www.ti.com/product/ADS8681)
- [TI E2E Support Forums](https://e2e.ti.com)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

This driver was developed based on:
- Official TI ADS8681 datasheet
- Arduino ADS8681 library by Sponge5
- Community discussions on TI E2E forums
