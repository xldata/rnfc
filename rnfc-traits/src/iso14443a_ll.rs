use core::convert::Infallible;
use core::fmt::Debug;

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Frame {
    Standard { timeout_1fc: u32 },
    WupA,
    ReqA,
    Anticoll { bits: usize },
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ErrorKind {
    Other,
    Timeout,
    Corruption,
}

pub trait Error: Debug {
    fn kind(&self) -> ErrorKind;
}

impl Error for ErrorKind {
    fn kind(&self) -> ErrorKind {
        *self
    }
}

impl Error for Infallible {
    fn kind(&self) -> ErrorKind {
        match *self {}
    }
}

pub trait Reader {
    type Error: Error;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8], opts: Frame) -> Result<usize, Self::Error>;
}

impl<T: Reader> Reader for &mut T {
    type Error = T::Error;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8], opts: Frame) -> Result<usize, Self::Error> {
        T::transceive(self, tx, rx, opts).await
    }
}
