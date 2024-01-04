
use embedded_hal::blocking::i2c::{Read, Write};
use sensirion_i2c::i2c;

/// SEN5x errors
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum Error<E> {
    /// I²C bus error
    #[cfg_attr(feature = "thiserror", error("I2C: {0}"))]
    I2c(E),
    /// CRC checksum validation failed
    #[cfg_attr(feature = "thiserror", error("CRC"))]
    Crc,
    /// Not allowed when periodic measurement is running
    #[cfg_attr(feature = "thiserror", error("Not Allowed"))]
    NotAllowed,
    /// Internal fail
    #[cfg_attr(feature = "thiserror", error("Internal"))]
    Internal
}

impl<E, I2cWrite, I2cRead> From<i2c::Error<I2cWrite, I2cRead>> for Error<E>
where
    I2cWrite: Write<Error = E>,
    I2cRead: Read<Error = E>,
{
    fn from(err: i2c::Error<I2cWrite, I2cRead>) -> Self {
        match err {
            i2c::Error::Crc => Error::Crc,
            i2c::Error::I2cWrite(e) => Error::I2c(e),
            i2c::Error::I2cRead(e) => Error::I2c(e),
        }
    }
}
