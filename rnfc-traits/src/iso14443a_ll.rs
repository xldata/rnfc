use core::future::Future;

pub struct TransceiveOptions<'a> {
    pub tx: &'a [u8],
    pub rx: &'a mut [u8],
    pub crc: bool,
    pub bits: usize,
    pub timeout_ms: u32,
}

pub struct TransceiveResult {
    pub bits: usize,
    pub collision: bool,
}

pub trait Reader {
    type Error;

    type TransceiveFuture<'a>: Future<Output = Result<TransceiveResult, Self::Error>>
    where
        Self: 'a;

    fn transceive<'a>(&'a mut self, opts: TransceiveOptions<'a>) -> Self::TransceiveFuture<'a>;
}

impl<T: Reader> Reader for &mut T {
    type Error = T::Error;

    type TransceiveFuture<'a> = T::TransceiveFuture<'a>
    where
        Self: 'a;

    fn transceive<'a>(&'a mut self, opts: TransceiveOptions<'a>) -> Self::TransceiveFuture<'a> {
        T::transceive(self, opts)
    }
}
