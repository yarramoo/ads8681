//! ADS8681 Driver
//!
//! Platform-agnostic Rust driver for the Texas Instruments ADS8681
//! 16-bit, 1 MSPS, Single-Supply SAR ADC with Programmable Bipolar Input Ranges
//!
//! This driver uses the `embedded-hal` traits for SPI communication,
//! making it portable across different embedded platforms.
//!
//! # Features
//! - Read 16-bit ADC conversions
//! - Configure programmable input ranges (±12.288V to ±2.56V bipolar, 0-12.288V to 0-5.12V unipolar)
//! - Read and write device registers
//! - Read device ID
//! - Configure alarms and thresholds
//!
//! # Example
//! ```no_run
//! use ads8681_driver::{ADS8681, InputRange};
//! # use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
//! # use embedded_hal_mock::eh1::digital::Mock as PinMock;
//! # let spi = SpiMock::new(&[]);
//! # let cs = PinMock::new(&[]);
//!
//! let mut adc = ADS8681::new(spi, cs);
//! adc.set_input_range(InputRange::BipolarThreeVref).unwrap();
//! let value = adc.read_adc().unwrap();
//! ```

#![no_std]

use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

/// SPI command opcodes for ADS8681
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    /// No operation
    Nop = 0b0000000,
    /// Clear half-word (16-bit)
    ClearHword = 0b1100000,
    /// Read half-word (16-bit) from register
    ReadHword = 0b1100100,
    /// Read full word from register
    Read = 0b0100100,
    /// Write full 16-bit word to register
    WriteFull = 0b1101000,
    /// Write most significant byte to register
    WriteMs = 0b1101001,
    /// Write least significant byte to register
    WriteLs = 0b1101010,
    /// Set half-word
    SetHword = 0b1101100,
}

/// Register addresses for ADS8681
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Register {
    /// Device ID register
    DeviceId = 0x00,
    /// Reset and power control register
    RstPwrctl = 0x04,
    /// SDI control register
    SdiCtl = 0x08,
    /// SDO control register
    SdoCtl = 0x0C,
    /// Data output control register
    DataoutCtl = 0x10,
    /// Range selection register
    RangeSel = 0x14,
    /// Alarm configuration register
    Alarm = 0x20,
    /// Alarm high threshold register
    AlarmHTh = 0x24,
    /// Alarm low threshold register
    AlarmLTh = 0x28,
}

/// Input range configurations
///
/// The ADS8681 supports both bipolar and unipolar input ranges.
/// Vref is typically 4.096V, giving actual voltage ranges:
/// - BipolarThreeVref: ±12.288V
/// - BipolarTwoPointFiveVref: ±10.24V
/// - BipolarOnePointFiveVref: ±6.144V
/// - BipolarOnePointTwoFiveVref: ±5.12V
/// - BipolarZeroPointSixTwoFiveVref: ±2.56V
/// - UnipolarThreeVref: 0 to 12.288V
/// - UnipolarTwoPointFiveVref: 0 to 10.24V
/// - UnipolarOnePointFiveVref: 0 to 6.144V
/// - UnipolarOnePointTwoFiveVref: 0 to 5.12V
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum InputRange {
    /// Bipolar ±3 × Vref (±12.288V with 4.096V reference)
    BipolarThreeVref = 0b0000,
    /// Bipolar ±2.5 × Vref (±10.24V with 4.096V reference)
    BipolarTwoPointFiveVref = 0b0001,
    /// Bipolar ±1.5 × Vref (±6.144V with 4.096V reference)
    BipolarOnePointFiveVref = 0b0010,
    /// Bipolar ±1.25 × Vref (±5.12V with 4.096V reference)
    BipolarOnePointTwoFiveVref = 0b0011,
    /// Bipolar ±0.625 × Vref (±2.56V with 4.096V reference)
    BipolarZeroPointSixTwoFiveVref = 0b0100,
    /// Unipolar 0 to 3 × Vref (0 to 12.288V with 4.096V reference)
    UnipolarThreeVref = 0b1000,
    /// Unipolar 0 to 2.5 × Vref (0 to 10.24V with 4.096V reference)
    UnipolarTwoPointFiveVref = 0b1001,
    /// Unipolar 0 to 1.5 × Vref (0 to 6.144V with 4.096V reference)
    UnipolarOnePointFiveVref = 0b1010,
    /// Unipolar 0 to 1.25 × Vref (0 to 5.12V with 4.096V reference)
    UnipolarOnePointTwoFiveVref = 0b1011,
}

/// Error types for ADS8681 operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    /// SPI communication error
    Spi(E),
    /// Invalid device ID
    InvalidDeviceId,
}

/// ADS8681 driver instance
pub struct ADS8681<SPI, CS> {
    spi: SPI,
    cs: CS,
}

impl<SPI, CS, E> ADS8681<SPI, CS>
where
    SPI: SpiDevice<Error = E>,
    CS: OutputPin,
{
    /// Create a new ADS8681 driver instance
    ///
    /// # Arguments
    /// * `spi` - SPI peripheral implementing the embedded-hal SpiDevice trait
    /// * `cs` - Chip select pin implementing the OutputPin trait
    pub fn new(spi: SPI, cs: CS) -> Self {
        Self { spi, cs }
    }

    /// Initialize the device and verify communication
    ///
    /// Reads the device ID register to verify proper communication
    pub fn init(&mut self) -> Result<u16, Error<E>> {
        let device_id = self.read_register(Register::DeviceId)?;

        // Device ID for ADS8681 should be 0x0681
        if device_id != 0x0681 {
            return Err(Error::InvalidDeviceId);
        }

        Ok(device_id)
    }

    /// Read a 16-bit value from the ADC
    ///
    /// This performs a conversion and returns the 16-bit result.
    /// The value is a signed or unsigned 16-bit integer depending on
    /// the configured input range (bipolar vs unipolar).
    pub fn read_adc(&mut self) -> Result<u16, Error<E>> {
        // Send NOP command to trigger conversion and read previous result
        let response = self.send_command(Command::Nop, 0x00, 0x0000)?;

        // ADC data is in the upper 16 bits of the response
        Ok((response >> 16) as u16)
    }

    /// Set the input range configuration
    ///
    /// # Arguments
    /// * `range` - Desired input range from the InputRange enum
    pub fn set_input_range(&mut self, range: InputRange) -> Result<(), Error<E>> {
        self.write_register(Register::RangeSel, range as u16)
    }

    /// Get the current input range configuration
    pub fn get_input_range(&mut self) -> Result<InputRange, Error<E>> {
        let value = self.read_register(Register::RangeSel)?;

        // Map the register value back to InputRange enum
        match value & 0x0F {
            0b0000 => Ok(InputRange::BipolarThreeVref),
            0b0001 => Ok(InputRange::BipolarTwoPointFiveVref),
            0b0010 => Ok(InputRange::BipolarOnePointFiveVref),
            0b0011 => Ok(InputRange::BipolarOnePointTwoFiveVref),
            0b0100 => Ok(InputRange::BipolarZeroPointSixTwoFiveVref),
            0b1000 => Ok(InputRange::UnipolarThreeVref),
            0b1001 => Ok(InputRange::UnipolarTwoPointFiveVref),
            0b1010 => Ok(InputRange::UnipolarOnePointFiveVref),
            0b1011 => Ok(InputRange::UnipolarOnePointTwoFiveVref),
            _ => Ok(InputRange::BipolarThreeVref), // Default fallback
        }
    }

    /// Read a 16-bit register value
    ///
    /// # Arguments
    /// * `reg` - Register to read from
    pub fn read_register(&mut self, reg: Register) -> Result<u16, Error<E>> {
        // Send READ_HWORD command
        self.send_command(Command::ReadHword, reg as u16, 0x0000)?;

        // Send NOP to retrieve the register data
        let response = self.send_command(Command::Nop, 0x00, 0x0000)?;

        // Register data is in the upper 16 bits
        Ok((response >> 16) as u16)
    }

    /// Write a 16-bit value to a register
    ///
    /// # Arguments
    /// * `reg` - Register to write to
    /// * `value` - 16-bit value to write
    pub fn write_register(&mut self, reg: Register, value: u16) -> Result<(), Error<E>> {
        self.send_command(Command::WriteFull, reg as u16, value)?;
        Ok(())
    }

    /// Reset the device
    pub fn reset(&mut self) -> Result<(), Error<E>> {
        // Write to RST_PWRCTL register with reset bit set
        self.write_register(Register::RstPwrctl, 0x8000)
    }

    /// Set alarm high threshold
    ///
    /// # Arguments
    /// * `threshold` - 16-bit threshold value
    pub fn set_alarm_high_threshold(&mut self, threshold: u16) -> Result<(), Error<E>> {
        self.write_register(Register::AlarmHTh, threshold)
    }

    /// Set alarm low threshold
    ///
    /// # Arguments
    /// * `threshold` - 16-bit threshold value
    pub fn set_alarm_low_threshold(&mut self, threshold: u16) -> Result<(), Error<E>> {
        self.write_register(Register::AlarmLTh, threshold)
    }

    /// Get alarm high threshold
    pub fn get_alarm_high_threshold(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::AlarmHTh)
    }

    /// Get alarm low threshold
    pub fn get_alarm_low_threshold(&mut self) -> Result<u16, Error<E>> {
        self.read_register(Register::AlarmLTh)
    }

    /// Send a 4-byte SPI command to the device and return the 4-byte response
    ///
    /// # Arguments
    /// * `cmd` - Command opcode
    /// * `addr` - Register address (9-bit)
    /// * `data` - 16-bit data payload
    ///
    /// # Returns
    /// 32-bit response from the device
    fn send_command(&mut self, cmd: Command, addr: u16, data: u16) -> Result<u32, Error<E>> {
        // Construct the 4-byte command packet
        // Byte 0: Command (7 bits) << 1 | Address MSB (bit 8)
        // Byte 1: Address LSB (bits 7-0)
        // Byte 2: Data MSB
        // Byte 3: Data LSB
        let mut tx_buf = [0u8; 4];
        let mut rx_buf = [0u8; 4];

        tx_buf[0] = ((cmd as u8) << 1) | (((addr >> 8) & 0x01) as u8);
        tx_buf[1] = (addr & 0xFF) as u8;
        tx_buf[2] = ((data >> 8) & 0xFF) as u8;
        tx_buf[3] = (data & 0xFF) as u8;

        // Perform SPI transaction
        self.spi.transfer(&mut rx_buf, &tx_buf).map_err(Error::Spi)?;

        // Assemble 32-bit response
        let response = ((rx_buf[0] as u32) << 24)
            | ((rx_buf[1] as u32) << 16)
            | ((rx_buf[2] as u32) << 8)
            | (rx_buf[3] as u32);

        Ok(response)
    }

    /// Consume the driver and return the SPI and CS pin
    pub fn release(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }
}

/// Convert a raw ADC reading to voltage
///
/// # Arguments
/// * `raw_value` - Raw 16-bit ADC reading
/// * `range` - Input range configuration used
/// * `vref` - Reference voltage (typically 4.096V)
///
/// # Returns
/// Voltage in volts
pub fn raw_to_voltage(raw_value: u16, range: InputRange, vref: f32) -> f32 {
    match range {
        // Bipolar ranges - interpret as signed 16-bit
        InputRange::BipolarThreeVref => {
            let signed = raw_value as i16;
            (signed as f32 / 32768.0) * 3.0 * vref
        }
        InputRange::BipolarTwoPointFiveVref => {
            let signed = raw_value as i16;
            (signed as f32 / 32768.0) * 2.5 * vref
        }
        InputRange::BipolarOnePointFiveVref => {
            let signed = raw_value as i16;
            (signed as f32 / 32768.0) * 1.5 * vref
        }
        InputRange::BipolarOnePointTwoFiveVref => {
            let signed = raw_value as i16;
            (signed as f32 / 32768.0) * 1.25 * vref
        }
        InputRange::BipolarZeroPointSixTwoFiveVref => {
            let signed = raw_value as i16;
            (signed as f32 / 32768.0) * 0.625 * vref
        }
        // Unipolar ranges - interpret as unsigned 16-bit
        InputRange::UnipolarThreeVref => {
            (raw_value as f32 / 65536.0) * 3.0 * vref
        }
        InputRange::UnipolarTwoPointFiveVref => {
            (raw_value as f32 / 65536.0) * 2.5 * vref
        }
        InputRange::UnipolarOnePointFiveVref => {
            (raw_value as f32 / 65536.0) * 1.5 * vref
        }
        InputRange::UnipolarOnePointTwoFiveVref => {
            (raw_value as f32 / 65536.0) * 1.25 * vref
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Import vec! macro and other std features for tests
    extern crate std;
    use std::vec;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use embedded_hal_mock::eh1::digital::Mock as PinMock;

    #[test]
    fn test_command_encoding() {
        let spi = SpiMock::new(&[
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(
                vec![0b00000000, 0x14, 0x00, 0x00],
                vec![0x00, 0x00, 0xAB, 0xCD],
            ),
            SpiTransaction::transaction_end(),
        ]);
        let cs = PinMock::new(&[]);

        let mut adc = ADS8681::new(spi, cs);

        // Send NOP command to address 0x14 with no data
        let result = adc.send_command(Command::Nop, 0x14, 0x0000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0x0000ABCD);

        let (mut spi, mut cs) = adc.release();
        spi.done();
        cs.done();
    }

    #[test]
    fn test_read_register_command_sequence() {
        let spi = SpiMock::new(&[
            // First transaction: READ_HWORD command
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(
                vec![0b11001000, 0x00, 0x00, 0x00], // READ_HWORD to DeviceId (0x00)
                vec![0x00, 0x00, 0x00, 0x00],
            ),
            SpiTransaction::transaction_end(),
            // Second transaction: NOP to retrieve data
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(
                vec![0b00000000, 0x00, 0x00, 0x00], // NOP
                vec![0x06, 0x81, 0x00, 0x00], // Device ID in upper 16 bits
            ),
            SpiTransaction::transaction_end(),
        ]);
        let cs = PinMock::new(&[]);

        let mut adc = ADS8681::new(spi, cs);
        let device_id = adc.read_register(Register::DeviceId);

        assert!(device_id.is_ok());
        assert_eq!(device_id.unwrap(), 0x0681);

        let (mut spi, mut cs) = adc.release();
        spi.done();
        cs.done();
    }

    #[test]
    fn test_write_register() {
        let spi = SpiMock::new(&[
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(
                vec![0b11010000, 0x14, 0x00, 0x03], // WRITE_FULL to RangeSel with value 0x0003
                vec![0x00, 0x00, 0x00, 0x00],
            ),
            SpiTransaction::transaction_end(),
        ]);
        let cs = PinMock::new(&[]);

        let mut adc = ADS8681::new(spi, cs);
        let result = adc.write_register(Register::RangeSel, 0x0003);

        assert!(result.is_ok());

        let (mut spi, mut cs) = adc.release();
        spi.done();
        cs.done();
    }

    #[test]
    fn test_set_input_range() {
        let spi = SpiMock::new(&[
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer(
                vec![0b11010000, 0x14, 0x00, 0x00], // WRITE_FULL to RangeSel
                vec![0x00, 0x00, 0x00, 0x00],
            ),
            SpiTransaction::transaction_end(),
        ]);
        let cs = PinMock::new(&[]);

        let mut adc = ADS8681::new(spi, cs);
        let result = adc.set_input_range(InputRange::BipolarThreeVref);

        assert!(result.is_ok());

        let (mut spi, mut cs) = adc.release();
        spi.done();
        cs.done();
    }

    #[test]
    fn test_voltage_conversion_bipolar() {
        let vref = 4.096;

        // Test positive full scale (±12.288V range)
        let voltage = raw_to_voltage(0x7FFF, InputRange::BipolarThreeVref, vref);
        assert!((voltage - 12.287).abs() < 0.01);

        // Test negative full scale
        let voltage = raw_to_voltage(0x8000, InputRange::BipolarThreeVref, vref);
        assert!((voltage + 12.288).abs() < 0.01);

        // Test zero
        let voltage = raw_to_voltage(0x0000, InputRange::BipolarThreeVref, vref);
        assert!(voltage.abs() < 0.01);
    }

    #[test]
    fn test_voltage_conversion_unipolar() {
        let vref = 4.096;

        // Test full scale (0-12.288V range)
        let voltage = raw_to_voltage(0xFFFF, InputRange::UnipolarThreeVref, vref);
        assert!((voltage - 12.288).abs() < 0.01);

        // Test zero
        let voltage = raw_to_voltage(0x0000, InputRange::UnipolarThreeVref, vref);
        assert!(voltage.abs() < 0.01);

        // Test mid-scale
        let voltage = raw_to_voltage(0x8000, InputRange::UnipolarThreeVref, vref);
        assert!((voltage - 6.144).abs() < 0.01);
    }
}
