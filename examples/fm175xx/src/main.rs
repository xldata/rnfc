#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// Must go FIRST so that other mods see its macros.
mod fmt;

use core::fmt::Debug;

use cortex_m::asm::delay;
use embassy_executor::executor::Spawner;
use embassy_nrf::config::LfclkSource;
use embassy_nrf::gpio::{Flex, Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{interrupt, pac, Peripherals};
use embedded_hal::digital::blocking::OutputPin;
use embedded_hal::spi::blocking::{SpiBusFlush, SpiDevice};
use embedded_hal::spi::{Error, ErrorKind, ErrorType};
use rnfc::iso14443a::Poller;
use rnfc::iso_dep::IsoDepA;
use rnfc_fm175xx::{Fm175xx, I2cInterface, WakeupConfig};
use rnfc_traits::iso_dep::Reader;
use {defmt_rtt as _, panic_probe as _};

fn config() -> embassy_nrf::config::Config {
    let mut config = embassy_nrf::config::Config::default();
    //config.hfclk_source = HfclkSource::ExternalXtal;
    config.lfclk_source = LfclkSource::ExternalXtal;
    config
}

#[embassy_executor::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    unsafe {
        let nvmc = &*pac::NVMC::ptr();
        let power = &*pac::POWER::ptr();

        // SET NFCPINS = 0 to use them as GPIOs
        if *(0x1000120C as *mut u32) & 1 != 0 {
            nvmc.config.write(|w| w.wen().wen());
            while nvmc.ready.read().ready().is_busy() {}
            core::ptr::write_volatile(0x1000120C as *mut u32, !1);
            while nvmc.ready.read().ready().is_busy() {}
            nvmc.config.reset();
            while nvmc.ready.read().ready().is_busy() {}
            cortex_m::peripheral::SCB::sys_reset();
        }

        /*
        // SET PSELRESET
        const RESET_PIN: u32 = 21;
        if *(0x10001200 as *mut u32) != RESET_PIN || *(0x10001204 as *mut u32) != RESET_PIN {
            nvmc.config.write(|w| w.wen().wen());
            while nvmc.ready.read().ready().is_busy() {}
            core::ptr::write_volatile(0x10001200 as *mut u32, RESET_PIN);
            while nvmc.ready.read().ready().is_busy() {}
            core::ptr::write_volatile(0x10001204 as *mut u32, RESET_PIN);
            while nvmc.ready.read().ready().is_busy() {}
            nvmc.config.reset();
            while nvmc.ready.read().ready().is_busy() {}
            cortex_m::peripheral::SCB::sys_reset();
        }
         */

        // Enable DC-DC
        power.dcdcen.write(|w| w.dcdcen().enabled());

        // Enable flash cache
        nvmc.icachecnf.write(|w| w.cacheen().enabled());
    }

    let npd = p.P0_15;
    let mut scl = p.P0_20;
    let mut sda = p.P0_22;
    let irq = p.P0_24;

    let npd = Output::new(npd, Level::High, OutputDrive::Standard);
    let irq = Input::new(irq, Pull::None);

    {
        // Try to unstick the i2c bus if it's stuck.
        let mut scl = Flex::new(&mut scl);
        scl.set_high();
        scl.set_as_input_output(Pull::None, OutputDrive::HighDrive0Disconnect1);
        let mut sda = Flex::new(&mut sda);
        sda.set_high();
        sda.set_as_input_output(Pull::None, OutputDrive::HighDrive0Disconnect1);

        if sda.is_low() {
            warn!("SDA stuck low.")
        }
        if scl.is_low() {
            warn!("SCL stuck low.")
        }

        info!("wiggling SCL...");
        for _ in 0..12 {
            scl.set_low();
            cortex_m::asm::delay(64_000_000 / 100_000 / 2);
            scl.set_high();
            cortex_m::asm::delay(64_000_000 / 100_000 / 2);

            if scl.is_low() {
                warn!("SCL still low while clocking it.")
            }
        }

        if sda.is_low() {
            warn!("SDA still stuck low.")
        }
        if scl.is_low() {
            warn!("SCL still stuck low.")
        }

        info!("doing start+stop");
        cortex_m::asm::delay(64_000_000 / 100_000 / 2);
        sda.set_low();
        cortex_m::asm::delay(64_000_000 / 100_000 / 2);
        sda.set_high();
        cortex_m::asm::delay(64_000_000 / 100_000 / 2);

        if sda.is_low() {
            warn!("SDA STILL stuck low, wtf?")
        }
        if scl.is_low() {
            warn!("SCL STILL stuck low, wtf?")
        }
    }

    let mut config = twim::Config::default();
    config.frequency = twim::Frequency::K400;
    config.scl_high_drive = true;
    config.sda_high_drive = true;

    let twim = Twim::new(
        p.TWISPI0,
        interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0),
        sda,
        scl,
        config,
    );

    let iface = I2cInterface::new(twim, 0x28);
    let mut fm = Fm175xx::new(iface, npd, irq).await;

    let wup_config = WakeupConfig {
        sleep_time: 2,
        prepare_time: 13,
        measure_time: 11,
        threshold: 20,
        n_drive: 1,
        p_drive: 4,
    };

    loop {
        fm.wait_for_card(wup_config).await.unwrap();

        let poller = fm.start_iso14443a().await.unwrap();
        let mut poller = Poller::new(poller);

        /*
        for card in poller.search::<8>().await.unwrap() {
            info!("got card: {:02x}", card);
        }
         */

        match poller.select_any().await {
            Err(e) => warn!("poll failed: {:?}", e),
            Ok(card) => {
                let mut isodep = IsoDepA::new(card).await.unwrap();
                let mut rx = [0; 256];
                let tx = [0x90, 0x60, 0x00, 0x00, 0x00];
                match isodep.transceive(&tx, &mut rx).await {
                    Err(e) => warn!("transceive failed: {:?}", e),
                    Ok(n) => info!("rxd: {:02x}", &rx[..n]),
                }
            }
        };
    }

    /*

    fm.clear_fifo();
    info!("fifo level: {}", fm.regs().fifolevel().read().0);
    fm.write_fifo(&[0x12, 0x34, 0x56, 0x78]);
    info!("fifo level: {}", fm.regs().fifolevel().read().0);

    let mut buf = [0; 1];
    fm.read_fifo(&mut buf);
    info!("fifo level: {}", fm.regs().fifolevel().read().0);
    let mut buf = [0; 3];
    fm.read_fifo(&mut buf);
    info!("fifo level: {}", fm.regs().fifolevel().read().0);


    cs.set_low();

    let mut buf = [0x82, 0x00];
    info!("buf: {:x}", buf);
    spi.transfer(&mut buf).unwrap();
    info!("buf: {:x}", buf);
    cs.set_high();

    loop {
        info!("HIGH");
        led.set_high();
        Timer::after(Duration::from_millis(300)).await;
        info!("LOW");
        led.set_low();
        Timer::after(Duration::from_millis(300)).await;
    }
     */
}

/// Error type for [`ExclusiveDevice`] operations.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ExclusiveDeviceError<BUS, CS> {
    /// An inner SPI bus operation failed
    Spi(BUS),
    /// Asserting or deasserting CS failed
    Cs(CS),
}

impl<BUS, CS> Error for ExclusiveDeviceError<BUS, CS>
where
    BUS: Error + Debug,
    CS: Debug,
{
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Spi(e) => e.kind(),
            Self::Cs(_) => ErrorKind::ChipSelectFault,
        }
    }
}

/// [`SpiDevice`] implementation with exclusive access to the bus (not shared).
///
/// This is the most straightforward way of obtaining an [`SpiDevice`] from an [`SpiBus`],
/// ideal for when no sharing is required (only one SPI device is present on the bus).
pub struct ExclusiveDevice<BUS, CS> {
    bus: BUS,
    cs: CS,
}

impl<BUS, CS> ExclusiveDevice<BUS, CS> {
    /// Create a new ExclusiveDevice
    pub fn new(bus: BUS, cs: CS) -> Self {
        Self { bus, cs }
    }
}

impl<BUS, CS> ErrorType for ExclusiveDevice<BUS, CS>
where
    BUS: ErrorType,
    CS: OutputPin,
{
    type Error = ExclusiveDeviceError<BUS::Error, CS::Error>;
}

impl<BUS, CS> SpiDevice for ExclusiveDevice<BUS, CS>
where
    BUS: SpiBusFlush,
    CS: OutputPin,
{
    type Bus = BUS;

    fn transaction<R>(
        &mut self,
        f: impl FnOnce(&mut Self::Bus) -> Result<R, <Self::Bus as ErrorType>::Error>,
    ) -> Result<R, Self::Error> {
        self.cs.set_low().map_err(ExclusiveDeviceError::Cs)?;

        delay(10_000);

        let f_res = f(&mut self.bus);

        // On failure, it's important to still flush and deassert CS.
        let flush_res = self.bus.flush();

        delay(10_000);
        let cs_res = self.cs.set_high();

        let f_res = f_res.map_err(ExclusiveDeviceError::Spi)?;
        flush_res.map_err(ExclusiveDeviceError::Spi)?;
        cs_res.map_err(ExclusiveDeviceError::Cs)?;

        Ok(f_res)
    }
}
