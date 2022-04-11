use core::fmt::Debug;
use cortex_m::asm::delay;
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

    fn read_reg_raw(&mut self, reg: u8) -> u8
    where
        <T as Write<u8>>::Error: Debug,
        <T as Transfer<u8>>::Error: Debug,
        C::Error: Debug,
    {
        delay(10_000);

        self.cs.set_low().unwrap();
        delay(10_000);

        let mut buf = [0x80 | (reg << 1), 0x00];
        self.spi.transfer(&mut buf).unwrap();
        let res = buf[1];

        delay(10_000);

        self.cs.set_high().unwrap();

        //trace!("         read_raw {=u8:02x} = {=u8:02x}", reg, res);
        res
    }

    fn write_reg_raw(&mut self, reg: u8, val: u8)
    where
        <T as Write<u8>>::Error: Debug,
        <T as Transfer<u8>>::Error: Debug,
        C::Error: Debug,
    {
        //trace!("         write_raw {=u8:02x} = {=u8:02x}", reg, val);
        delay(10_000);

        self.cs.set_low().unwrap();
        delay(10_000);

        let buf = [(reg << 1), val];
        self.spi.write(&buf).unwrap();

        delay(10_000);

        self.cs.set_high().unwrap();
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
    fn read_reg(&mut self, reg: usize) -> u8 {
        let reg = reg as u8;
        let res = if reg < 0x40 {
            // Main register
            self.read_reg_raw(reg)
        } else {
            // Extended register
            let reg = reg - 0x40;
            self.write_reg_raw(0x0f, reg | 0x80);
            self.read_reg_raw(0x0f) & 0x3F
        };

        trace!("     read {=u8:02x} = {=u8:02x}", reg, res);
        res
    }

    fn write_reg(&mut self, reg: usize, val: u8) {
        let reg = reg as u8;
        trace!("     write {=u8:02x} = {=u8:02x}", reg, val);

        if reg < 0x40 {
            // Main register
            self.write_reg_raw(reg, val)
        } else {
            // Extended register
            let reg = reg - 0x40;
            self.write_reg_raw(0x0F, reg | 0x40);
            self.write_reg_raw(0x0F, (val & 0x3F) | 0xC0);
        }
    }

    fn read_fifo(&mut self, data: &mut [u8]) {
        delay(10_000);
        self.cs.set_low().unwrap();

        delay(10_000);

        self.spi.write(&[0x92]).unwrap();
        data.fill(0x92);
        data[data.len() - 1] = 0x80;
        self.spi.transfer(data).unwrap();
        delay(10_000);

        self.cs.set_high().unwrap();

        trace!("     read_fifo {=[u8]:02x}", data);
    }

    fn write_fifo(&mut self, data: &[u8]) {
        trace!("     write_fifo {=[u8]:02x}", data);
        delay(10_000);
        self.cs.set_low().unwrap();
        delay(10_000);

        self.spi.write(&[0x12]).unwrap();
        self.spi.write(data).unwrap();
        delay(10_000);

        self.cs.set_high().unwrap();
    }
}
