use embedded_hal::spi::{Operation, SpiDevice};

use super::Interface;

pub struct SpiInterface<T: SpiDevice> {
    spi: T,
}

impl<T: SpiDevice> SpiInterface<T> {
    pub fn new(spi: T) -> Self {
        Self { spi }
    }
}

impl<T: SpiDevice> Interface for SpiInterface<T> {
    type Error = T::Error;

    fn do_command(&mut self, cmd: u8) -> Result<(), Self::Error> {
        trace!("     cmd {:02x}", cmd);

        let buf = [cmd];
        self.spi.write(&buf)
    }

    fn read_reg(&mut self, reg: u8) -> Result<u8, Self::Error> {
        let res = match reg {
            // Register space A
            0x00..=0x3F => {
                let mut buf = [0x40 | reg, 0x00];
                self.spi.transfer_in_place(&mut buf)?;
                buf[1]
            }
            // Register space B
            0x40..=0x7F => {
                let mut buf = [0xFB, 0x40 | (reg - 0x40), 0x00];
                self.spi.transfer_in_place(&mut buf)?;
                buf[2]
            }
            // Register space Test
            0x80..=0xBF => {
                let mut buf = [0xFC, 0x40 | (reg - 0x80), 0x00];
                self.spi.transfer_in_place(&mut buf)?;
                buf[2]
            }
            _ => panic!("Invalid reg {}", reg),
        };

        trace!("     read {:02x} = {:02x}", reg, res);
        Ok(res)
    }

    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), Self::Error> {
        trace!("     write {:02x} = {:02x}", reg, val);

        match reg {
            // Register space A
            0x00..=0x3F => {
                let buf = [reg, val];
                self.spi.write(&buf)
            }
            // Register space B
            0x40..=0x7F => {
                let buf = [0xFB, reg - 0x40, val];
                self.spi.write(&buf)
            }
            // Register space Test
            0x80..=0xBF => {
                let buf = [0xFC, reg - 0x80, val];
                self.spi.write(&buf)
            }
            _ => panic!("Invalid reg {}", reg),
        }
    }

    fn read_fifo(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.transaction(&mut [Operation::Write(&[0x9f]), Operation::Read(data)])
    }

    fn write_fifo(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.spi.transaction(&mut [Operation::Write(&[0x80]), Operation::Write(data)])
    }
}
