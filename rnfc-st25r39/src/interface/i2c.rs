use embedded_hal::i2c::I2c;

use super::Interface;

pub struct I2cInterface<T>
where
    T: I2c,
{
    i2c: T,
    address: u8,
}

impl<T> I2cInterface<T>
where
    T: I2c,
{
    pub fn new(i2c: T, address: u8) -> Self {
        Self { i2c, address }
    }
}

impl<T> Interface for I2cInterface<T>
where
    T: I2c,
{
    type Error = T::Error;

    fn do_command(&mut self, cmd: u8) -> Result<(), Self::Error> {
        trace!("     cmd {=u8:x}", cmd);
        self.i2c.write(self.address, &[cmd])
    }

    fn read_reg(&mut self, reg: u8) -> Result<u8, Self::Error> {
        let mut buf = [0x00];

        match reg {
            // Register space A
            0x00..=0x3F => self.i2c.write_read(self.address, &[0x40 | reg], &mut buf)?,
            // Register space B
            0x40..=0x7F => self.i2c.write_read(self.address, &[0xFB, 0x40 | (reg - 0x40)], &mut buf)?,
            // Register space Test
            0x80..=0xBF => self.i2c.write_read(self.address, &[0xFC, 0x40 | (reg - 0x80)], &mut buf)?,
            _ => panic!("Invalid reg {}", reg),
        };

        let res = buf[0];

        trace!("     read {=u8:x} = {=u8:x}", reg, res);
        Ok(res)
    }

    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), Self::Error> {
        trace!("     write {=u8:x} = {=u8:x}", reg, val);

        match reg {
            // Register space A
            0x00..=0x3F => self.i2c.write(self.address, &[reg, val]),
            // Register space B
            0x40..=0x7F => self.i2c.write(self.address, &[0xFB, reg - 0x40, val]),
            // Register space Test
            0x80..=0xBF => self.i2c.write(self.address, &[0xFC, reg - 0x80, val]),
            _ => panic!("Invalid reg {}", reg),
        }
    }

    fn read_fifo(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(self.address, &[0x9F], data)
    }

    fn write_fifo(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        let mut buf = [0u8; 512 + 1];
        buf[0] = 0x80;
        buf[1..][..data.len()].copy_from_slice(data);
        self.i2c.write(self.address, &buf[..1 + data.len()])
    }
}
