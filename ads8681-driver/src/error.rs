//! Error types for the ADS8681 driver

/// Error types that can occur during ADS8681 operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    /// SPI communication error from the underlying bus
    Spi(E),

    /// Invalid device ID detected during initialization
    ///
    /// Expected device ID is 0x0681. If a different value is read,
    /// this error is returned, indicating possible communication issues
    /// or wrong device.
    InvalidDeviceId,
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::Spi(error)
    }
}

#[cfg(feature = "defmt")]
impl<E> defmt::Format for Error<E>
where
    E: defmt::Format,
{
    fn format(&self, f: defmt::Formatter) {
        match self {
            Error::Spi(e) => defmt::write!(f, "SPI error: {}", e),
            Error::InvalidDeviceId => defmt::write!(f, "Invalid device ID"),
        }
    }
}
