//! Mock SPI Verification Example
//!
//! This example demonstrates how to use the ADS8681 driver with mock SPI
//! for testing and verification without actual hardware.
//!
//! Run with: cargo run --example mock_verification

use ads8681_driver::{ADS8681, InputRange, raw_to_voltage};
use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
use embedded_hal_mock::eh1::digital::Mock as PinMock;

fn main() {
    println!("ADS8681 Driver - Mock Verification Example");
    println!("===========================================\n");

    // Test 1: Device Initialization
    println!("Test 1: Device Initialization");
    test_init();
    println!();

    // Test 2: Set Input Range
    println!("Test 2: Set Input Range");
    test_set_range();
    println!();

    // Test 3: Read ADC Value
    println!("Test 3: Read ADC Value");
    test_read_adc();
    println!();

    // Test 4: Configure Alarms
    println!("Test 4: Configure Alarms");
    test_alarms();
    println!();

    // Test 5: Voltage Conversion
    println!("Test 5: Voltage Conversion");
    test_voltage_conversion();
    println!();

    println!("All verification tests completed successfully!");
}

fn test_init() {
    // Setup mock SPI with expected device ID response
    let expectations = [
        // READ_HWORD command to DeviceId register
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11001000, 0x00, 0x00, 0x00],
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // NOP command to retrieve device ID
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b00000000, 0x00, 0x00, 0x00],
            vec![0x06, 0x81, 0x00, 0x00], // Device ID: 0x0681
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);

    match adc.init() {
        Ok(device_id) => {
            println!("  ✓ Device initialized successfully");
            println!("  ✓ Device ID: 0x{:04X}", device_id);
            assert_eq!(device_id, 0x0681);
        }
        Err(_) => {
            println!("  ✗ Failed to initialize device");
            panic!("Initialization failed");
        }
    }

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}

fn test_set_range() {
    // Setup mock SPI for range configuration
    let expectations = [
        // WRITE_FULL to RangeSel register
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11010000, 0x14, 0x00, 0x01], // Set to BipolarTwoPointFiveVref (±10.24V)
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);

    match adc.set_input_range(InputRange::BipolarTwoPointFiveVref) {
        Ok(_) => {
            println!("  ✓ Input range set to ±10.24V (±2.5 × Vref)");
        }
        Err(_) => {
            println!("  ✗ Failed to set input range");
            panic!("Set range failed");
        }
    }

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}

fn test_read_adc() {
    // Setup mock SPI for ADC reading
    let expectations = [
        // NOP command to read ADC value
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b00000000, 0x00, 0x00, 0x00],
            vec![0x7F, 0xFF, 0x00, 0x00], // ADC value: 0x7FFF (positive full-scale)
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);

    match adc.read_adc() {
        Ok(value) => {
            println!("  ✓ ADC value read: 0x{:04X} ({})", value, value);
            assert_eq!(value, 0x7FFF);

            // Convert to voltage (assuming ±12.288V range)
            let voltage = raw_to_voltage(value, InputRange::BipolarThreeVref, 4.096);
            println!("  ✓ Voltage: {:.3}V", voltage);
        }
        Err(_) => {
            println!("  ✗ Failed to read ADC value");
            panic!("ADC read failed");
        }
    }

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}

fn test_alarms() {
    // Setup mock SPI for alarm configuration
    let expectations = [
        // Set high threshold
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11010000, 0x24, 0x60, 0x00], // AlarmHTh = 0x6000
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // Set low threshold
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11010000, 0x28, 0x20, 0x00], // AlarmLTh = 0x2000
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
    ];

    let spi = SpiMock::new(&expectations);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);

    match adc.set_alarm_high_threshold(0x6000) {
        Ok(_) => println!("  ✓ High threshold set to 0x6000"),
        Err(_) => panic!("Failed to set high threshold"),
    }

    match adc.set_alarm_low_threshold(0x2000) {
        Ok(_) => println!("  ✓ Low threshold set to 0x2000"),
        Err(_) => panic!("Failed to set low threshold"),
    }

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}

fn test_voltage_conversion() {
    let vref = 4.096;

    println!("  Testing voltage conversions with Vref = {}V:", vref);
    println!();

    // Bipolar ±12.288V range
    println!("  Bipolar ±12.288V range:");
    let ranges_bipolar = [
        (0x7FFF, "Positive full-scale"),
        (0x4000, "Positive half-scale"),
        (0x0000, "Zero"),
        (0xC000, "Negative half-scale"),
        (0x8000, "Negative full-scale"),
    ];

    for (raw, desc) in ranges_bipolar.iter() {
        let voltage = raw_to_voltage(*raw, InputRange::BipolarThreeVref, vref);
        println!("    0x{:04X} ({:20}): {:+.3}V", raw, desc, voltage);
    }
    println!();

    // Unipolar 0-12.288V range
    println!("  Unipolar 0-12.288V range:");
    let ranges_unipolar = [
        (0xFFFF, "Full-scale"),
        (0xC000, "75% scale"),
        (0x8000, "Half-scale"),
        (0x4000, "25% scale"),
        (0x0000, "Zero"),
    ];

    for (raw, desc) in ranges_unipolar.iter() {
        let voltage = raw_to_voltage(*raw, InputRange::UnipolarThreeVref, vref);
        println!("    0x{:04X} ({:20}): {:.3}V", raw, desc, voltage);
    }

    println!();
    println!("  ✓ All voltage conversions computed successfully");
}
