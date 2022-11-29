pub trait Reader {
    type Error;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, Self::Error>;
}

impl<T: Reader> Reader for &mut T {
    type Error = T::Error;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, Self::Error> {
        T::transceive(self, tx, rx).await
    }
}
