use embedded_hal_async::spi;
#[derive(Debug)]
pub enum Error {
    LateCollision,
    Spi(&'static str),
    Gpio,
    IOError(&'static str),
    ConfigError
}

#[derive(Debug)]
pub enum BuilderError {
    SpiNotSpecified,
    CsnNotSpecified,
    CeNotSpecified,
    TimerNotSpecified,
    Other
}

impl spi::Error for Error {
    fn kind(&self) -> spi::ErrorKind {
        spi::ErrorKind::Other
    }
}