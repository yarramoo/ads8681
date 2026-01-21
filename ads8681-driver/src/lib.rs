//! # ADS8681 Driver
//!
//! Platform-agnostic Rust driver for the Texas Instruments ADS8681
//! 16-bit, 1 MSPS, Single-Supply SAR ADC with Programmable Bipolar Input Ranges.
//!
//! This driver uses the `embedded-hal` traits for SPI communication,
//! making it portable across different embedded platforms.
//!
//! ## Features
//!
//! - Read 16-bit ADC conversions
//! - Configure programmable input ranges (±12.288V to ±2.56V bipolar, 0-12.288V to 0-5.12V unipolar)
//! - Read and write device registers
//! - Verify device communication via device ID check
//! - Configure alarm thresholds
//! - Convert raw ADC values to voltages
//! - `no_std` compatible
//! - Comprehensive test coverage with mock SPI
//!
//! ## Quick Start
//!
//! ```no_run
//! use ads8681_driver::{ADS8681, InputRange, raw_to_voltage};
//! # use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
//! # use embedded_hal_mock::eh1::digital::Mock as PinMock;
//! # let spi = SpiMock::new(&[]);
//! # let cs = PinMock::new(&[]);
//!
//! // Create driver instance
//! let mut adc = ADS8681::new(spi, cs);
//!
//! // Initialize and verify communication (returns device ID 0x0681)
//! // adc.init().expect("Failed to initialize");
//!
//! // Configure input range
//! // adc.set_input_range(InputRange::BipolarThreeVref)
//! //     .expect("Failed to set range");
//!
//! // Read ADC value
//! // let raw = adc.read_adc().expect("Failed to read ADC");
//! // let voltage = raw_to_voltage(raw, InputRange::BipolarThreeVref, 4.096);
//! ```
//!
//! ## SPI Configuration
//!
//! The ADS8681 SPI interface requires:
//! - **SPI Mode**: MODE_0 (CPOL=0, CPHA=0)
//! - **Clock Speed**: Up to 1 MHz
//! - **Bit Order**: MSB first
//! - **Frame Size**: 4 bytes (32 bits)
//!
//! ## Module Organization
//!
//! - [`error`] - Error types
//! - [`register`] - Register definitions and constants
//! - [`command`] - SPI command encoding/decoding
//! - [`range`] - Input range configuration and voltage conversion

#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]

use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

// Public modules
pub mod command;
pub mod error;
pub mod range;
pub mod register;

// Private test module
#[cfg(test)]
mod tests;

// Re-export commonly used types
pub use command::Command;
pub use error::Error;
pub use range::{raw_to_voltage, voltage_to_raw, InputRange, STANDARD_VREF};
pub use register::{Register, DEVICE_ID};

/// ADS8681 driver instance
///
/// This is the main driver struct that provides an interface to the ADS8681 ADC.
/// It uses generic types for the SPI bus and chip select pin to support any
/// platform that implements the `embedded-hal` traits.
///
/// # Type Parameters
///
/// * `SPI` - SPI device implementing the `SpiDevice` trait
/// * `CS` - Chip select pin implementing the `OutputPin` trait (currently unused but reserved)
///
/// # Examples
///
/// ```no_run
/// use ads8681_driver::ADS8681;
/// # use embedded_hal_mock::eh1::spi::Mock as SpiMock;
/// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
/// # let spi = SpiMock::new(&[]);
/// # let cs_pin = PinMock::new(&[]);
///
/// let mut adc = ADS8681::new(spi, cs_pin);
/// ```
pub struct ADS8681<SPI, CS> {
    spi: SPI,
    cs: CS, // Reserved for future use with manual CS control
}

impl<SPI, CS, E> ADS8681<SPI, CS>
where
    SPI: SpiDevice<Error = E>,
    CS: OutputPin,
{
    /// Create a new ADS8681 driver instance
    ///
    /// Creates a new driver instance without performing any initialization.
    /// Call [`init()`](Self::init) after creation to verify device communication.
    ///
    /// # Arguments
    ///
    /// * `spi` - SPI device configured for the ADS8681 (MODE_0, up to 1 MHz)
    /// * `cs` - Chip select pin (reserved for future manual CS control)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ads8681_driver::ADS8681;
    /// # use embedded_hal_mock::eh1::spi::Mock as SpiMock;
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # let spi = SpiMock::new(&[]);
    /// # let cs_pin = PinMock::new(&[]);
    ///
    /// let adc = ADS8681::new(spi, cs_pin);
    /// ```
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self { spi, cs }
    }

    /// Initialize the device and verify communication
    ///
    /// Reads the device ID register and verifies it matches the expected value (0x0681).
    /// This confirms proper SPI communication with the ADS8681.
    ///
    /// # Returns
    ///
    /// * `Ok(device_id)` - Device ID (should be 0x0681)
    /// * `Err(Error::InvalidDeviceId)` - If device ID doesn't match
    /// * `Err(Error::Spi(e))` - If SPI communication fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ads8681_driver::ADS8681;
    /// # use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # let spi = SpiMock::new(&[
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0xC8, 0x00, 0x00, 0x00], vec![0, 0, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0x00, 0x00, 0x00, 0x00], vec![0x06, 0x81, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// # ]);
    /// # let cs = PinMock::new(&[]);
    /// let mut adc = ADS8681::new(spi, cs);
    ///
    /// match adc.init() {
    ///     Ok(id) => println!("Device ID: 0x{:04X}", id),
    ///     Err(_) => println!("Failed to initialize"),
    /// }
    /// ```
    pub fn init(&mut self) -> Result<u16, Error<E>> {
        let device_id = self.read_register(Register::DeviceId)?;

        if device_id != DEVICE_ID {
            return Err(Error::InvalidDeviceId);
        }

        Ok(device_id)
    }

    /// Read a 16-bit value from the ADC
    ///
    /// Performs a conversion and returns the 16-bit result. The interpretation of this
    /// value (signed vs unsigned) depends on the configured input range:
    /// - Bipolar ranges: Interpret as `i16` (two's complement)
    /// - Unipolar ranges: Interpret as `u16`
    ///
    /// Use [`raw_to_voltage()`] to convert the raw value to a voltage.
    ///
    /// # Returns
    ///
    /// * `Ok(u16)` - 16-bit ADC conversion result
    /// * `Err(Error::Spi(e))` - If SPI communication fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ads8681_driver::{ADS8681, InputRange, raw_to_voltage};
    /// # use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # let spi = SpiMock::new(&[
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0x00, 0x00, 0x00, 0x00], vec![0x7F, 0xFF, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// # ]);
    /// # let cs = PinMock::new(&[]);
    /// # let mut adc = ADS8681::new(spi, cs);
    /// let raw = adc.read_adc().unwrap();
    /// let voltage = raw_to_voltage(raw, InputRange::BipolarThreeVref, 4.096);
    /// ```
    pub fn read_adc(&mut self) -> Result<u16, Error<E>> {
        // Send NOP command to trigger conversion and read previous result
        let response = self.send_command(Command::Nop, 0x00, 0x0000)?;

        // ADC data is in the upper 16 bits of the response
        Ok(command::extract_adc_data(response))
    }

    /// Set the input range configuration
    ///
    /// Configures the ADC input range. The range determines the full-scale voltage
    /// and whether the output is signed (bipolar) or unsigned (unipolar).
    ///
    /// # Arguments
    ///
    /// * `range` - Desired input range from the [`InputRange`] enum
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ads8681_driver::{ADS8681, InputRange};
    /// # use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # let spi = SpiMock::new(&[
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0xD0, 0x14, 0x00, 0x01], vec![0, 0, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// # ]);
    /// # let cs = PinMock::new(&[]);
    /// # let mut adc = ADS8681::new(spi, cs);
    /// // Set to ±10.24V range
    /// adc.set_input_range(InputRange::BipolarTwoPointFiveVref).unwrap();
    /// ```
    pub fn set_input_range(&mut self, range: InputRange) -> Result<(), Error<E>> {
        self.write_register(Register::RangeSel, range.value())
    }

    /// Get the current input range configuration
    ///
    /// Reads the RANGE_SEL register and returns the configured input range.
    /// If the register contains an invalid value, returns the default range
    /// (BipolarThreeVref).
    ///
    /// # Returns
    ///
    /// The currently configured [`InputRange`]
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ads8681_driver::{ADS8681, InputRange};
    /// # use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # let spi = SpiMock::new(&[
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0xC8, 0x14, 0x00, 0x00], vec![0, 0, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0x00, 0x00, 0x00, 0x00], vec![0x00, 0x01, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// # ]);
    /// # let cs = PinMock::new(&[]);
    /// # let mut adc = ADS8681::new(spi, cs);
    /// let range = adc.get_input_range().unwrap();
    /// println!("Current range: {}", range.name());
    /// ```
    pub fn get_input_range(&mut self) -> Result<InputRange, Error<E>> {
        let value = self.read_register(Register::RangeSel)?;

        // Convert register value to InputRange, defaulting to BipolarThreeVref if invalid
        Ok(InputRange::from_register_value(value).unwrap_or(InputRange::BipolarThreeVref))
    }

    /// Read a 16-bit value from a register
    ///
    /// Reads a register using the two-step process:
    /// 1. Send READ_HWORD command with register address
    /// 2. Send NOP command to retrieve the register data
    ///
    /// # Arguments
    ///
    /// * `reg` - Register to read from
    ///
    /// # Returns
    ///
    /// * `Ok(u16)` - 16-bit register value
    /// * `Err(Error::Spi(e))` - If SPI communication fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ads8681_driver::{ADS8681, Register};
    /// # use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # let spi = SpiMock::new(&[
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0xC8, 0x00, 0x00, 0x00], vec![0, 0, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0x00, 0x00, 0x00, 0x00], vec![0x06, 0x81, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// # ]);
    /// # let cs = PinMock::new(&[]);
    /// # let mut adc = ADS8681::new(spi, cs);
    /// let device_id = adc.read_register(Register::DeviceId).unwrap();
    /// ```
    pub fn read_register(&mut self, reg: Register) -> Result<u16, Error<E>> {
        // Step 1: Send READ_HWORD command
        self.send_command(Command::ReadHword, reg.address(), 0x0000)?;

        // Step 2: Send NOP to retrieve the register data
        let response = self.send_command(Command::Nop, 0x00, 0x0000)?;

        // Register data is in the upper 16 bits
        Ok(command::extract_register_data(response))
    }

    /// Write a 16-bit value to a register
    ///
    /// Writes a value to the specified register using the WRITE_FULL command.
    ///
    /// # Arguments
    ///
    /// * `reg` - Register to write to
    /// * `value` - 16-bit value to write
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Write successful
    /// * `Err(Error::Spi(e))` - If SPI communication fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ads8681_driver::{ADS8681, Register};
    /// # use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # let spi = SpiMock::new(&[
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0xD0, 0x14, 0x00, 0x03], vec![0, 0, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// # ]);
    /// # let cs = PinMock::new(&[]);
    /// # let mut adc = ADS8681::new(spi, cs);
    /// adc.write_register(Register::RangeSel, 0x0003).unwrap();
    /// ```
    pub fn write_register(&mut self, reg: Register, value: u16) -> Result<(), Error<E>> {
        self.send_command(Command::WriteFull, reg.address(), value)?;
        Ok(())
    }

    /// Reset the device
    ///
    /// Performs a software reset by writing to the RST_PWRCTL register.
    /// After reset, all registers return to their default values.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ads8681_driver::ADS8681;
    /// # use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # let spi = SpiMock::new(&[
    /// #     SpiTransaction::transaction_start(),
    /// #     SpiTransaction::transfer(vec![0xD0, 0x04, 0x80, 0x00], vec![0, 0, 0, 0]),
    /// #     SpiTransaction::transaction_end(),
    /// # ]);
    /// # let cs = PinMock::new(&[]);
    /// # let mut adc = ADS8681::new(spi, cs);
    /// adc.reset().unwrap();
    /// ```
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        self.write_register(Register::RstPwrctl, register::RST_BIT)
    }

    /// Set the alarm high threshold
    ///
    /// Configures the upper threshold for the alarm comparator.
    ///
    /// # Arguments
    ///
    /// * `threshold` - 16-bit threshold value
    pub fn set_alarm_high_threshold(&mut self, threshold: u16) -> Result<(), Error<E>> {
        self.write_register(Register::AlarmHTh, threshold)
    }

    /// Set the alarm low threshold
    ///
    /// Configures the lower threshold for the alarm comparator.
    ///
    /// # Arguments
    ///
    /// * `threshold` - 16-bit threshold value
    pub fn set_alarm_low_threshold(&mut self, threshold: u16) -> Result<(), Error<E>> {
        self.write_register(Register::AlarmLTh, threshold)
    }

    /// Get the alarm high threshold
    ///
    /// Reads the current high threshold value.
    pub fn get_alarm_high_threshold(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::AlarmHTh)
    }

    /// Get the alarm low threshold
    ///
    /// Reads the current low threshold value.
    pub fn get_alarm_low_threshold(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::AlarmLTh)
    }

    /// Send a 4-byte SPI command and receive a 4-byte response
    ///
    /// Low-level SPI transaction method. Encodes a command and performs the SPI transfer.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command opcode
    /// * `addr` - Register address (9-bit)
    /// * `data` - 16-bit data payload
    ///
    /// # Returns
    ///
    /// 32-bit response from the device
    fn send_command(&mut self, cmd: Command, addr: u16, data: u16) -> Result<u32, Error<E>> {
        let tx_buf = command::encode_command(cmd, addr, data);
        let mut rx_buf = [0u8; 4];

        // Perform SPI transaction (CS is handled automatically by SpiDevice)
        self.spi.transfer(&mut rx_buf, &tx_buf).map_err(Error::Spi)?;

        // Decode and return response
        Ok(command::decode_response(&rx_buf))
    }

    /// Consume the driver and return the SPI bus and CS pin
    ///
    /// This allows you to reuse the SPI peripheral and GPIO pin for other purposes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ads8681_driver::ADS8681;
    /// # use embedded_hal_mock::eh1::spi::Mock as SpiMock;
    /// # use embedded_hal_mock::eh1::digital::Mock as PinMock;
    /// # let spi = SpiMock::new(&[]);
    /// # let cs = PinMock::new(&[]);
    /// let adc = ADS8681::new(spi, cs);
    /// // ... use the ADC ...
    /// let (spi, cs) = adc.release();
    /// // ... use spi and cs for something else ...
    /// ```
    pub fn release(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }
}
