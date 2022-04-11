//mod i2c;
mod spi;

//pub use i2c::I2cInterface;
pub use spi::SpiInterface;

pub trait Interface {
    fn read_reg(&mut self, reg: usize) -> u8;
    fn write_reg(&mut self, reg: usize, val: u8);

    fn read_fifo(&mut self, data: &mut [u8]);
    fn write_fifo(&mut self, data: &[u8]);
}
