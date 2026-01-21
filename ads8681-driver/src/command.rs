//! SPI command definitions for the ADS8681 ADC
//!
//! The ADS8681 uses a 4-byte SPI command structure:
//! - Byte 0: [Command (7 bits)] [Address bit 8]
//! - Byte 1: Address bits [7:0]
//! - Byte 2: Data MSB
//! - Byte 3: Data LSB
//!
//! Responses also use 4 bytes, with ADC data or register values
//! typically in the upper 16 bits.

/// SPI command opcodes for ADS8681
///
/// These commands control register access and ADC operation.
/// Commands are 7 bits and occupy the upper portion of the first byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    /// No operation
    ///
    /// Used to trigger a conversion and read the previous result,
    /// or to complete a register read sequence.
    Nop = 0b0000000,

    /// Clear half-word (16-bit)
    ///
    /// Clears the specified register to 0x0000.
    ClearHword = 0b1100000,

    /// Read half-word (16-bit) from register
    ///
    /// Initiates a register read. The actual data is returned
    /// during the next SPI transaction (typically with a NOP command).
    ReadHword = 0b1100100,

    /// Read full word from register
    ///
    /// Alternative read command for compatibility with some modes.
    Read = 0b0100100,

    /// Write full 16-bit word to register
    ///
    /// Writes both bytes of the data field to the specified register.
    WriteFull = 0b1101000,

    /// Write most significant byte to register
    ///
    /// Writes only the upper byte (byte 2) to the register.
    WriteMs = 0b1101001,

    /// Write least significant byte to register
    ///
    /// Writes only the lower byte (byte 3) to the register.
    WriteLs = 0b1101010,

    /// Set half-word
    ///
    /// Sets specified bits in the register (OR operation).
    SetHword = 0b1101100,
}

impl Command {
    /// Get the command opcode as a u8
    pub const fn opcode(self) -> u8 {
        self as u8
    }

    /// Check if this command writes to registers
    pub const fn is_write(self) -> bool {
        matches!(
            self,
            Command::WriteFull | Command::WriteMs | Command::WriteLs | Command::SetHword
        )
    }

    /// Check if this command reads from registers
    pub const fn is_read(self) -> bool {
        matches!(self, Command::ReadHword | Command::Read)
    }

    /// Get the command name as a string
    pub const fn name(self) -> &'static str {
        match self {
            Command::Nop => "NOP",
            Command::ClearHword => "CLEAR_HWORD",
            Command::ReadHword => "READ_HWORD",
            Command::Read => "READ",
            Command::WriteFull => "WRITE_FULL",
            Command::WriteMs => "WRITE_MS",
            Command::WriteLs => "WRITE_LS",
            Command::SetHword => "SET_HWORD",
        }
    }
}

/// Encode a command into a 4-byte SPI packet
///
/// # Arguments
/// * `cmd` - Command opcode
/// * `addr` - Register address (9-bit, though only lower bits typically used)
/// * `data` - 16-bit data payload
///
/// # Returns
/// 4-byte array ready to transmit via SPI
pub fn encode_command(cmd: Command, addr: u16, data: u16) -> [u8; 4] {
    [
        ((cmd as u8) << 1) | (((addr >> 8) & 0x01) as u8),
        (addr & 0xFF) as u8,
        ((data >> 8) & 0xFF) as u8,
        (data & 0xFF) as u8,
    ]
}

/// Decode a 4-byte SPI response into a 32-bit value
///
/// # Arguments
/// * `response` - 4-byte response buffer from SPI
///
/// # Returns
/// 32-bit assembled response value
pub fn decode_response(response: &[u8; 4]) -> u32 {
    ((response[0] as u32) << 24)
        | ((response[1] as u32) << 16)
        | ((response[2] as u32) << 8)
        | (response[3] as u32)
}

/// Extract ADC data from a response
///
/// ADC conversion data is typically in the upper 16 bits of the response.
///
/// # Arguments
/// * `response` - 32-bit response value
///
/// # Returns
/// 16-bit ADC data
pub fn extract_adc_data(response: u32) -> u16 {
    (response >> 16) as u16
}

/// Extract register data from a response
///
/// Register data is typically in the upper 16 bits of the response,
/// returned after a NOP following a READ_HWORD command.
///
/// # Arguments
/// * `response` - 32-bit response value
///
/// # Returns
/// 16-bit register data
pub fn extract_register_data(response: u32) -> u16 {
    (response >> 16) as u16
}
