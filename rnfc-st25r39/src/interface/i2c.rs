use embedded_hal::blocking::i2c::{Write, WriteRead};

use super::Interface;

pub struct I2cInterface<T>
where
    T: WriteRead + Write,
{
    i2c: T,
    address: u8,
}

impl<T> I2cInterface<T>
where
    T: WriteRead + Write,
{
    pub fn new(i2c: T, address: u8) -> Self {
        Self { i2c, address }
    }
}

impl<T> Interface for I2cInterface<T>
where
    T: WriteRead + Write,
    <T as Write>::Error: defmt::Format,
    <T as WriteRead>::Error: defmt::Format,
{
    fn do_command(&mut self, cmd: u8) {
        trace!("     cmd {=u8:x}", cmd);
        unwrap!(self.i2c.write(self.address, &[cmd]))
    }

    fn read_reg(&mut self, reg: u8) -> u8 {
        let mut buf = [0x00];

        match reg {
            // Register space A
            0x00..=0x3F => unwrap!(self.i2c.write_read(self.address, &[0x40 | reg], &mut buf)),
            // Register space B
            0x40..=0x7F => unwrap!(self.i2c.write_read(self.address, &[0xFB, 0x40 | (reg - 0x40)], &mut buf)),
            // Register space Test
            0x80..=0xBF => unwrap!(self.i2c.write_read(self.address, &[0xFC, 0x40 | (reg - 0x80)], &mut buf)),
            _ => panic!("Invalid reg {}", reg),
        };

        let res = buf[0];

        trace!("     read {=u8:x} = {=u8:x}", reg, res);
        res
    }

    fn write_reg(&mut self, reg: u8, val: u8) {
        trace!("     write {=u8:x} = {=u8:x}", reg, val);

        match reg {
            // Register space A
            0x00..=0x3F => unwrap!(self.i2c.write(self.address, &[reg, val])),
            // Register space B
            0x40..=0x7F => unwrap!(self.i2c.write(self.address, &[0xFB, reg - 0x40, val])),
            // Register space Test
            0x80..=0xBF => unwrap!(self.i2c.write(self.address, &[0xFC, reg - 0x80, val])),
            _ => panic!("Invalid reg {}", reg),
        }
    }

    fn read_fifo(&mut self, data: &mut [u8]) {
        unwrap!(self.i2c.write_read(self.address, &[0x9F], data))
    }

    fn write_fifo(&mut self, data: &[u8]) {
        let mut buf = [0u8; 512 + 1];
        buf[0] = 0x80;
        buf[1..][..data.len()].copy_from_slice(data);
        unwrap!(self.i2c.write(self.address, &buf[..1 + data.len()]))
    }
}
