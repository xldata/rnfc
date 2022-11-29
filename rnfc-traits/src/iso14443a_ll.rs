#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Frame {
    Standard { timeout_ms: u32 },
    WupA,
    ReqA,
    Anticoll { bits: usize },
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ErrorKind {
    Other,
    NoResponse,
}

pub trait Error {
    fn kind(&self) -> ErrorKind;
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
