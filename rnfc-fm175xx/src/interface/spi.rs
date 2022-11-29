use cortex_m::asm::delay;
use embedded_hal::spi::{SpiBus, SpiBusWrite, SpiDevice};

use super::Interface;

pub struct SpiInterface<T: SpiDevice>
where
    T::Bus: SpiBus,
{
    spi: T,
}

impl<T: SpiDevice> SpiInterface<T>
where
    T::Bus: SpiBus,
{
    pub fn new(spi: T) -> Self {
        Self { spi }
    }

    fn read_reg_raw(&mut self, reg: u8) -> u8 {
        delay(10_000);

        let mut buf = [0x80 | (reg << 1), 0x00];
        self.spi.transfer_in_place(&mut buf).unwrap();
        let res = buf[1];

        //trace!("         read_raw {=u8:02x} = {=u8:02x}", reg, res);
        res
    }

    fn write_reg_raw(&mut self, reg: u8, val: u8) {
        //trace!("         write_raw {=u8:02x} = {=u8:02x}", reg, val);
        delay(10_000);

        let buf = [(reg << 1), val];
        self.spi.write(&buf).unwrap();
    }
}

impl<T: SpiDevice> Interface for SpiInterface<T>
where
    T::Bus: SpiBus,
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
        if data.len() == 0 {
            return;
        }

        delay(10_000);

        self.spi
            .transaction(|bus| {
                bus.write(&[0x92])?;
                data.fill(0x92);
                data[data.len() - 1] = 0x80;
                bus.transfer_in_place(data)?;
                Ok(())
            })
            .unwrap();

        trace!("     read_fifo {=[u8]:02x}", data);
    }

    fn write_fifo(&mut self, data: &[u8]) {
        trace!("     write_fifo {=[u8]:02x}", data);
        delay(10_000);

        self.spi
            .transaction(|bus| {
                bus.write(&[0x12])?;
                bus.write(data)?;
                Ok(())
            })
            .unwrap();
    }
}
