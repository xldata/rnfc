mod i2c;
mod spi;

pub use i2c::I2cInterface;
pub use spi::SpiInterface;

pub trait Interface {
    fn do_command(&mut self, cmd: u8);
    fn read_reg(&mut self, reg: u8) -> u8;
    fn write_reg(&mut self, reg: u8, val: u8);

    fn read_fifo(&mut self, data: &mut [u8]);
    fn write_fifo(&mut self, data: &[u8]);
}
