mod i2c;
mod spi;

use core::fmt::Debug;

pub use i2c::I2cInterface;
pub use spi::SpiInterface;

pub trait Interface {
    type Error: Debug;

    fn do_command(&mut self, cmd: u8) -> Result<(), Self::Error>;
    fn read_reg(&mut self, reg: u8) -> Result<u8, Self::Error>;
    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), Self::Error>;
    fn read_fifo(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
    fn write_fifo(&mut self, data: &[u8]) -> Result<(), Self::Error>;
}
