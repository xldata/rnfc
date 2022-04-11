use core::future::Future;

pub trait Reader {
    type Error;

    type TransceiveFuture<'a>: Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn transceive<'a>(&'a mut self, tx: &'a [u8], rx: &'a mut [u8]) -> Self::TransceiveFuture<'a>;
}

impl<T: Reader> Reader for &mut T {
    type Error = T::Error;

    type TransceiveFuture<'a> = T::TransceiveFuture<'a> where Self: 'a;

    fn transceive<'a>(&'a mut self, tx: &'a [u8], rx: &'a mut [u8]) -> Self::TransceiveFuture<'a> {
        T::transceive(self, tx, rx)
    }
}
