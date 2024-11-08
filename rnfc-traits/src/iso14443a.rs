pub use crate::iso14443a_ll::Error;

pub const UID_MAX_LEN: usize = 10;

pub trait Reader {
    type Error: Error;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8], timeout_1fc: u32) -> Result<usize, Self::Error>;

    fn uid(&self) -> &[u8];
    fn atqa(&self) -> [u8; 2];
    fn sak(&self) -> u8;
}

impl<T: Reader> Reader for &mut T {
    type Error = T::Error;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8], timeout_1fc: u32) -> Result<usize, Self::Error> {
        T::transceive(self, tx, rx, timeout_1fc).await
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
