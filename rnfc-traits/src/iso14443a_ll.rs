use core::future::Future;

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

    type TransceiveFuture<'a>: Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn transceive<'a>(&'a mut self, tx: &'a [u8], rx: &'a mut [u8], opts: Frame) -> Self::TransceiveFuture<'a>;
}

impl<T: Reader> Reader for &mut T {
    type Error = T::Error;

    type TransceiveFuture<'a> = T::TransceiveFuture<'a>
    where
        Self: 'a;

    fn transceive<'a>(&'a mut self, tx: &'a [u8], rx: &'a mut [u8], opts: Frame) -> Self::TransceiveFuture<'a> {
        T::transceive(self, tx, rx, opts)
    }
}
