//! Register definitions for the ADS8681 ADC
//!
//! The ADS8681 has 9 internal registers accessible via SPI.
//! All registers are 16 bits wide.

/// Register addresses for ADS8681
///
/// Each register controls different aspects of the ADC operation.
/// Registers are accessed using the SPI command protocol with
/// READ_HWORD and WRITE_FULL commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Register {
    /// Device ID register (Read-only)
    ///
    /// Returns 0x0681 for ADS8681. Used to verify device communication.
    DeviceId = 0x00,

    /// Reset and power control register
    ///
    /// Bit 15: RST - Writing 1 triggers a software reset
    /// Other bits: Reserved
    RstPwrctl = 0x04,

    /// SDI control register
    ///
    /// Controls Serial Data Input behavior in various modes
    SdiCtl = 0x08,

    /// SDO control register
    ///
    /// Controls Serial Data Output timing and format
    SdoCtl = 0x0C,

    /// Data output control register
    ///
    /// Controls what data is output on SDO during conversions
    DataoutCtl = 0x10,

    /// Range selection register
    ///
    /// Bits [3:0]: Range selection (see InputRange enum)
    /// Bits [15:4]: Reserved
    RangeSel = 0x14,

    /// Alarm configuration register
    ///
    /// Configures alarm comparator behavior
    Alarm = 0x20,

    /// Alarm high threshold register
    ///
    /// 16-bit threshold value for high alarm
    AlarmHTh = 0x24,

    /// Alarm low threshold register
    ///
    /// 16-bit threshold value for low alarm
    AlarmLTh = 0x28,
}

impl Register {
    /// Get the register address as a u16
    pub const fn address(self) -> u16 {
        self as u16
    }

    /// Check if this register is read-only
    pub const fn is_read_only(self) -> bool {
        matches!(self, Register::DeviceId)
    }

    /// Get the register name as a string
    pub const fn name(self) -> &'static str {
        match self {
            Register::DeviceId => "DEVICE_ID",
            Register::RstPwrctl => "RST_PWRCTL",
            Register::SdiCtl => "SDI_CTL",
            Register::SdoCtl => "SDO_CTL",
            Register::DataoutCtl => "DATAOUT_CTL",
            Register::RangeSel => "RANGE_SEL",
            Register::Alarm => "ALARM",
            Register::AlarmHTh => "ALARM_H_TH",
            Register::AlarmLTh => "ALARM_L_TH",
        }
    }
}

/// Expected device ID value for ADS8681
pub const DEVICE_ID: u16 = 0x0681;

/// Bit mask for software reset in RST_PWRCTL register
pub const RST_BIT: u16 = 0x8000;

/// Bit mask for range selection in RANGE_SEL register
pub const RANGE_MASK: u16 = 0x000F;
