use core::future::Future;

pub const UID_MAX_LEN: usize = 10;

pub trait Reader {
    type Error;

    type TransceiveFuture<'a>: Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn transceive<'a>(&'a mut self, tx: &'a [u8], rx: &'a mut [u8]) -> Self::TransceiveFuture<'a>;

    fn uid(&self) -> &[u8];
    fn atqa(&self) -> [u8; 2];
    fn sak(&self) -> u8;
}

impl<T: Reader> Reader for &mut T {
    type Error = T::Error;

    type TransceiveFuture<'a> = T::TransceiveFuture<'a>
    where
        Self: 'a;

    fn transceive<'a>(&'a mut self, tx: &'a [u8], rx: &'a mut [u8]) -> Self::TransceiveFuture<'a> {
        T::transceive(self, tx, rx)
    }

    fn uid(&self) -> &[u8] {
        T::uid(self)
    }
    fn atqa(&self) -> [u8; 2] {
        T::atqa(self)
    }
    fn sak(&self) -> u8 {
        T::sak(self)
    }
}
