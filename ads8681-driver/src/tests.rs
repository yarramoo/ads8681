//! Integration tests for the ADS8681 driver
//!
//! These tests use mock SPI to verify driver behavior without hardware.

use super::*;
use command::Command;
use register::Register;
use range::InputRange;

// Import vec! macro and other std features for tests
extern crate std;
use embedded_hal_mock::eh1::digital::Mock as PinMock;
use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
use std::vec;

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
            vec![0x06, 0x81, 0x00, 0x00],       // Device ID in upper 16 bits
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
fn test_init_success() {
    let spi = SpiMock::new(&[
        // READ_HWORD command
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11001000, 0x00, 0x00, 0x00],
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // NOP to retrieve device ID
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b00000000, 0x00, 0x00, 0x00],
            vec![0x06, 0x81, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
    ]);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);
    let result = adc.init();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0x0681);

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}

#[test]
fn test_init_invalid_device_id() {
    let spi = SpiMock::new(&[
        // READ_HWORD command
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11001000, 0x00, 0x00, 0x00],
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // NOP with wrong device ID
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b00000000, 0x00, 0x00, 0x00],
            vec![0x12, 0x34, 0x00, 0x00], // Wrong ID
        ),
        SpiTransaction::transaction_end(),
    ]);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);
    let result = adc.init();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), error::Error::InvalidDeviceId);

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}

#[test]
fn test_read_adc() {
    let spi = SpiMock::new(&[
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b00000000, 0x00, 0x00, 0x00],
            vec![0x7F, 0xFF, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
    ]);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);
    let result = adc.read_adc();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0x7FFF);

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}

#[test]
fn test_alarm_thresholds() {
    let spi = SpiMock::new(&[
        // Set high threshold
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11010000, 0x24, 0x60, 0x00],
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // Set low threshold
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11010000, 0x28, 0x20, 0x00],
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // Read high threshold
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11001000, 0x24, 0x00, 0x00],
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b00000000, 0x00, 0x00, 0x00],
            vec![0x60, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        // Read low threshold
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11001000, 0x28, 0x00, 0x00],
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b00000000, 0x00, 0x00, 0x00],
            vec![0x20, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
    ]);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);

    // Set thresholds
    assert!(adc.set_alarm_high_threshold(0x6000).is_ok());
    assert!(adc.set_alarm_low_threshold(0x2000).is_ok());

    // Read back thresholds
    assert_eq!(adc.get_alarm_high_threshold().unwrap(), 0x6000);
    assert_eq!(adc.get_alarm_low_threshold().unwrap(), 0x2000);

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}

#[test]
fn test_get_input_range() {
    let spi = SpiMock::new(&[
        // Read range register
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11001000, 0x14, 0x00, 0x00],
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b00000000, 0x00, 0x00, 0x00],
            vec![0x00, 0x01, 0x00, 0x00], // BipolarTwoPointFiveVref
        ),
        SpiTransaction::transaction_end(),
    ]);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);
    let result = adc.get_input_range();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), InputRange::BipolarTwoPointFiveVref);

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}

#[test]
fn test_reset() {
    let spi = SpiMock::new(&[
        SpiTransaction::transaction_start(),
        SpiTransaction::transfer(
            vec![0b11010000, 0x04, 0x80, 0x00], // Write 0x8000 to RST_PWRCTL
            vec![0x00, 0x00, 0x00, 0x00],
        ),
        SpiTransaction::transaction_end(),
    ]);
    let cs = PinMock::new(&[]);

    let mut adc = ADS8681::new(spi, cs);
    let result = adc.reset();

    assert!(result.is_ok());

    let (mut spi, mut cs) = adc.release();
    spi.done();
    cs.done();
}
