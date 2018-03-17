#[derive(Debug)]
pub enum Error<I2CE> {
    /// I2c error passed up
    I2cError(I2CE),
    /// This is not an SGTL5000
    Identification,
}

impl<I2CE> From<I2CE> for Error<I2CE> {
    fn from(e: I2CE) -> Self {
        Error::I2cError(e)
    }
}
