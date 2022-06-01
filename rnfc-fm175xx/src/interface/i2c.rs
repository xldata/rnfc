use embedded_hal::i2c::blocking::I2c;

use super::Interface;

pub struct I2cInterface<T: I2c> {
    i2c: T,
    address: u8,
}

impl<T: I2c> I2cInterface<T> {
    pub fn new(i2c: T, address: u8) -> Self {
        Self { i2c, address }
    }
}

impl<T: I2c> Interface for I2cInterface<T> {
    fn read_reg(&mut self, reg: usize) -> u8 {
        let mut buf = [0; 1];
        self.i2c.write_read(self.address, &[reg as u8], &mut buf).unwrap();
        buf[0]
    }

    fn write_reg(&mut self, reg: usize, val: u8) {
        self.i2c.write(self.address, &[reg as u8, val]).unwrap();
    }

    fn read_fifo(&mut self, data: &mut [u8]) {
        self.i2c.write_read(self.address, &[0x09], data).unwrap();
    }

    fn write_fifo(&mut self, data: &[u8]) {
        let mut buf = [0; 65];
        buf[0] = 0x09;
        buf[1..1 + data.len()].copy_from_slice(data);
        self.i2c.write(self.address, &buf[..1 + data.len()]).unwrap();
    }
}
