use core::fmt::Debug;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use super::Interface;

pub struct SpiInterface<T, C>
where
    T: Transfer<u8> + Write<u8>,
    C: OutputPin,
{
    spi: T,
    cs: C,
}

impl<T, C> SpiInterface<T, C>
where
    T: Transfer<u8> + Write<u8>,
    C: OutputPin,
{
    pub fn new(spi: T, cs: C) -> Self {
        Self { spi, cs }
    }
}

impl<T, C> Interface for SpiInterface<T, C>
where
    T: Transfer<u8> + Write<u8>,
    <T as Write<u8>>::Error: Debug,
    <T as Transfer<u8>>::Error: Debug,
    C: OutputPin,
    C::Error: Debug,
{
    fn do_command(&mut self, cmd: u8) {
        trace!("     cmd {=u8:x}", cmd);

        self.cs.set_low().unwrap();
        let buf = [cmd];
        self.spi.write(&buf).unwrap();
        self.cs.set_high().unwrap();
    }

    fn read_reg(&mut self, reg: u8) -> u8 {
        self.cs.set_low().unwrap();

        let res = match reg {
            // Register space A
            0x00..=0x3F => {
                let mut buf = [0x40 | reg, 0x00];
                self.spi.transfer(&mut buf).unwrap();
                buf[1]
            }
            // Register space B
            0x40..=0x7F => {
                let mut buf = [0xFB, 0x40 | (reg - 0x40), 0x00];
                self.spi.transfer(&mut buf).unwrap();
                buf[2]
            }
            // Register space Test
            0x80..=0xBF => {
                let mut buf = [0xFC, 0x40 | (reg - 0x80), 0x00];
                self.spi.transfer(&mut buf).unwrap();
                buf[2]
            }
            _ => panic!("Invalid reg {}", reg),
        };

        self.cs.set_high().unwrap();

        trace!("     read {=u8:x} = {=u8:x}", reg, res);
        res
    }

    fn write_reg(&mut self, reg: u8, val: u8) {
        trace!("     write {=u8:x} = {=u8:x}", reg, val);

        self.cs.set_low().unwrap();

        match reg {
            // Register space A
            0x00..=0x3F => {
                let buf = [reg, val];
                self.spi.write(&buf).unwrap();
            }
            // Register space B
            0x40..=0x7F => {
                let buf = [0xFB, reg - 0x40, val];
                self.spi.write(&buf).unwrap();
            }
            // Register space Test
            0x80..=0xBF => {
                let buf = [0xFC, reg - 0x80, val];
                self.spi.write(&buf).unwrap();
            }
            _ => panic!("Invalid reg {}", reg),
        }

        self.cs.set_high().unwrap();
    }

    fn read_fifo(&mut self, data: &mut [u8]) {
        self.cs.set_low().unwrap();

        self.spi.write(&[0x9F]).unwrap();
        self.spi.transfer(data).unwrap();

        self.cs.set_high().unwrap();
    }

    fn write_fifo(&mut self, data: &[u8]) {
        self.cs.set_low().unwrap();

        self.spi.write(&[0x80]).unwrap();
        self.spi.write(data).unwrap();

        self.cs.set_high().unwrap();
    }
}
