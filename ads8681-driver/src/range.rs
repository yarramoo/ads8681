//! Input range configuration and voltage conversion
//!
//! The ADS8681 supports programmable input ranges from ±12.288V to ±2.56V
//! in bipolar mode, and 0-12.288V to 0-5.12V in unipolar mode.

/// Input range configurations for ADS8681
///
/// The ADS8681 supports both bipolar and unipolar input ranges.
/// With the typical 4.096V internal reference, the actual voltage ranges are:
///
/// **Bipolar Ranges (signed output):**
/// - `BipolarThreeVref`: ±12.288V (±3 × 4.096V)
/// - `BipolarTwoPointFiveVref`: ±10.24V (±2.5 × 4.096V)
/// - `BipolarOnePointFiveVref`: ±6.144V (±1.5 × 4.096V)
/// - `BipolarOnePointTwoFiveVref`: ±5.12V (±1.25 × 4.096V)
/// - `BipolarZeroPointSixTwoFiveVref`: ±2.56V (±0.625 × 4.096V)
///
/// **Unipolar Ranges (unsigned output):**
/// - `UnipolarThreeVref`: 0 to 12.288V (0 to 3 × 4.096V)
/// - `UnipolarTwoPointFiveVref`: 0 to 10.24V (0 to 2.5 × 4.096V)
/// - `UnipolarOnePointFiveVref`: 0 to 6.144V (0 to 1.5 × 4.096V)
/// - `UnipolarOnePointTwoFiveVref`: 0 to 5.12V (0 to 1.25 × 4.096V)
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

impl InputRange {
    /// Get the range value as a u16 for writing to the RANGE_SEL register
    pub const fn value(self) -> u16 {
        self as u16
    }

    /// Check if this range is bipolar (signed output)
    pub const fn is_bipolar(self) -> bool {
        matches!(
            self,
            InputRange::BipolarThreeVref
                | InputRange::BipolarTwoPointFiveVref
                | InputRange::BipolarOnePointFiveVref
                | InputRange::BipolarOnePointTwoFiveVref
                | InputRange::BipolarZeroPointSixTwoFiveVref
        )
    }

    /// Check if this range is unipolar (unsigned output)
    pub const fn is_unipolar(self) -> bool {
        !self.is_bipolar()
    }

    /// Get the Vref multiplier for this range
    ///
    /// Returns the factor by which Vref is multiplied to get the full-scale range.
    /// For bipolar ranges, this is the magnitude (absolute value).
    pub const fn vref_multiplier(self) -> f32 {
        match self {
            InputRange::BipolarThreeVref | InputRange::UnipolarThreeVref => 3.0,
            InputRange::BipolarTwoPointFiveVref | InputRange::UnipolarTwoPointFiveVref => 2.5,
            InputRange::BipolarOnePointFiveVref | InputRange::UnipolarOnePointFiveVref => 1.5,
            InputRange::BipolarOnePointTwoFiveVref | InputRange::UnipolarOnePointTwoFiveVref => {
                1.25
            }
            InputRange::BipolarZeroPointSixTwoFiveVref => 0.625,
        }
    }

    /// Get the range name as a string
    pub const fn name(self) -> &'static str {
        match self {
            InputRange::BipolarThreeVref => "±3×Vref",
            InputRange::BipolarTwoPointFiveVref => "±2.5×Vref",
            InputRange::BipolarOnePointFiveVref => "±1.5×Vref",
            InputRange::BipolarOnePointTwoFiveVref => "±1.25×Vref",
            InputRange::BipolarZeroPointSixTwoFiveVref => "±0.625×Vref",
            InputRange::UnipolarThreeVref => "0-3×Vref",
            InputRange::UnipolarTwoPointFiveVref => "0-2.5×Vref",
            InputRange::UnipolarOnePointFiveVref => "0-1.5×Vref",
            InputRange::UnipolarOnePointTwoFiveVref => "0-1.25×Vref",
        }
    }

    /// Try to create an InputRange from a register value
    ///
    /// # Arguments
    /// * `value` - The 4-bit range value from the RANGE_SEL register
    ///
    /// # Returns
    /// The corresponding InputRange, or None if the value is invalid
    pub const fn from_register_value(value: u16) -> Option<Self> {
        match value & 0x0F {
            0b0000 => Some(InputRange::BipolarThreeVref),
            0b0001 => Some(InputRange::BipolarTwoPointFiveVref),
            0b0010 => Some(InputRange::BipolarOnePointFiveVref),
            0b0011 => Some(InputRange::BipolarOnePointTwoFiveVref),
            0b0100 => Some(InputRange::BipolarZeroPointSixTwoFiveVref),
            0b1000 => Some(InputRange::UnipolarThreeVref),
            0b1001 => Some(InputRange::UnipolarTwoPointFiveVref),
            0b1010 => Some(InputRange::UnipolarOnePointFiveVref),
            0b1011 => Some(InputRange::UnipolarOnePointTwoFiveVref),
            _ => None,
        }
    }
}

/// Convert a raw ADC reading to voltage
///
/// This function handles both bipolar (signed) and unipolar (unsigned) conversions
/// based on the configured input range.
///
/// # Arguments
/// * `raw_value` - Raw 16-bit ADC reading
/// * `range` - Input range configuration that was active during conversion
/// * `vref` - Reference voltage in volts (typically 4.096V)
///
/// # Returns
/// Voltage in volts
///
/// # Examples
/// ```
/// use ads8681_driver::{InputRange, raw_to_voltage};
///
/// // Convert a positive full-scale reading in bipolar mode
/// let voltage = raw_to_voltage(0x7FFF, InputRange::BipolarThreeVref, 4.096);
/// assert!((voltage - 12.288).abs() < 0.001);
///
/// // Convert a full-scale reading in unipolar mode
/// let voltage = raw_to_voltage(0xFFFF, InputRange::UnipolarThreeVref, 4.096);
/// assert!((voltage - 12.288).abs() < 0.001);
/// ```
pub fn raw_to_voltage(raw_value: u16, range: InputRange, vref: f32) -> f32 {
    let multiplier = range.vref_multiplier();

    if range.is_bipolar() {
        // Bipolar: interpret as signed 16-bit (-32768 to 32767)
        let signed = raw_value as i16;
        (signed as f32 / 32768.0) * multiplier * vref
    } else {
        // Unipolar: interpret as unsigned 16-bit (0 to 65535)
        (raw_value as f32 / 65536.0) * multiplier * vref
    }
}

/// Convert a voltage to a raw ADC value
///
/// This is the inverse of `raw_to_voltage`, useful for setting alarm thresholds
/// or testing.
///
/// # Arguments
/// * `voltage` - Voltage in volts
/// * `range` - Input range configuration
/// * `vref` - Reference voltage in volts (typically 4.096V)
///
/// # Returns
/// Raw 16-bit ADC value, clamped to valid range
///
/// # Examples
/// ```
/// use ads8681_driver::{InputRange, voltage_to_raw};
///
/// // Convert +6.144V in bipolar ±12.288V range (exactly half scale)
/// let raw = voltage_to_raw(6.144, InputRange::BipolarThreeVref, 4.096);
/// assert_eq!(raw, 0x4000);
/// ```
pub fn voltage_to_raw(voltage: f32, range: InputRange, vref: f32) -> u16 {
    let multiplier = range.vref_multiplier();
    let full_scale = multiplier * vref;

    if range.is_bipolar() {
        // Bipolar: convert to signed 16-bit
        let normalized = voltage / full_scale;
        let clamped = normalized.clamp(-1.0, 1.0);
        let signed = (clamped * 32768.0) as i16;
        signed as u16
    } else {
        // Unipolar: convert to unsigned 16-bit
        let normalized = voltage / full_scale;
        let clamped = normalized.clamp(0.0, 1.0);
        (clamped * 65536.0) as u16
    }
}

/// Standard reference voltage for ADS8681 (in volts)
pub const STANDARD_VREF: f32 = 4.096;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_is_bipolar() {
        assert!(InputRange::BipolarThreeVref.is_bipolar());
        assert!(InputRange::BipolarZeroPointSixTwoFiveVref.is_bipolar());
        assert!(!InputRange::UnipolarThreeVref.is_bipolar());
    }

    #[test]
    fn test_range_is_unipolar() {
        assert!(InputRange::UnipolarThreeVref.is_unipolar());
        assert!(!InputRange::BipolarThreeVref.is_unipolar());
    }

    #[test]
    fn test_from_register_value() {
        assert_eq!(
            InputRange::from_register_value(0b0000),
            Some(InputRange::BipolarThreeVref)
        );
        assert_eq!(
            InputRange::from_register_value(0b1000),
            Some(InputRange::UnipolarThreeVref)
        );
        assert_eq!(InputRange::from_register_value(0b0101), None);
    }

    #[test]
    fn test_bipolar_voltage_conversion() {
        let vref = 4.096;

        // Positive full scale
        let v = raw_to_voltage(0x7FFF, InputRange::BipolarThreeVref, vref);
        assert!((v - 12.287).abs() < 0.01);

        // Negative full scale
        let v = raw_to_voltage(0x8000, InputRange::BipolarThreeVref, vref);
        assert!((v + 12.288).abs() < 0.01);

        // Zero
        let v = raw_to_voltage(0x0000, InputRange::BipolarThreeVref, vref);
        assert!(v.abs() < 0.01);
    }

    #[test]
    fn test_unipolar_voltage_conversion() {
        let vref = 4.096;

        // Full scale
        let v = raw_to_voltage(0xFFFF, InputRange::UnipolarThreeVref, vref);
        assert!((v - 12.288).abs() < 0.01);

        // Zero
        let v = raw_to_voltage(0x0000, InputRange::UnipolarThreeVref, vref);
        assert!(v.abs() < 0.01);

        // Mid-scale
        let v = raw_to_voltage(0x8000, InputRange::UnipolarThreeVref, vref);
        assert!((v - 6.144).abs() < 0.01);
    }

    #[test]
    fn test_voltage_to_raw_bipolar() {
        let vref = 4.096;

        // Positive half scale
        let raw = voltage_to_raw(6.144, InputRange::BipolarThreeVref, vref);
        assert_eq!(raw, 0x4000);

        // Zero
        let raw = voltage_to_raw(0.0, InputRange::BipolarThreeVref, vref);
        assert_eq!(raw, 0x0000);

        // Negative half scale
        let raw = voltage_to_raw(-6.144, InputRange::BipolarThreeVref, vref);
        assert_eq!(raw, 0xC000);
    }

    #[test]
    fn test_voltage_to_raw_unipolar() {
        let vref = 4.096;

        // Half scale
        let raw = voltage_to_raw(6.144, InputRange::UnipolarThreeVref, vref);
        assert_eq!(raw, 0x8000);

        // Zero
        let raw = voltage_to_raw(0.0, InputRange::UnipolarThreeVref, vref);
        assert_eq!(raw, 0x0000);
    }

    #[test]
    fn test_voltage_clamping() {
        let vref = 4.096;

        // Over range should clamp to max
        let raw = voltage_to_raw(100.0, InputRange::BipolarThreeVref, vref);
        assert_eq!(raw, 0x7FFF);

        // Under range should clamp to min
        let raw = voltage_to_raw(-100.0, InputRange::BipolarThreeVref, vref);
        assert_eq!(raw, 0x8000);
    }
}
