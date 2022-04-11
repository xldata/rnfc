#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

// Must go FIRST so that other mods see its macros.
mod fmt;

use cortex_m::asm::delay;
use defmt_rtt as _;
use embassy::time::{Duration, Timer};
use embassy_nrf::config::LfclkSource;
use embassy_nrf::twim::{self, Twim};
use futures::future::pending;
use panic_probe as _;
use rnfc_fm175xx::{Fm175xx, SpiInterface};

use embassy::executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::pac;
use embassy_nrf::spim::{Config, Frequency, Spim};
use embassy_nrf::{interrupt, Peripherals};

#[embassy::task]
async fn watchdog_task(pin: AnyPin) {
    let mut out = Output::new(pin, Level::Low, OutputDrive::Standard);
    loop {
        out.set_high();
        delay(100);
        out.set_low();
        Timer::after(Duration::from_millis(800)).await;
    }
}

fn config() -> embassy_nrf::config::Config {
    let mut config = embassy_nrf::config::Config::default();
    //config.hfclk_source = HfclkSource::ExternalXtal;
    config.lfclk_source = LfclkSource::ExternalXtal;
    config
}

#[embassy::main(config = "config()")]
async fn main(spawner: Spawner, p: Peripherals) {
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

    spawner.spawn(watchdog_task(p.P0_04.degrade())).unwrap();

    let mut led_red = Output::new(p.P0_22, Level::High, OutputDrive::Standard);
    let mut led_green = Output::new(p.P0_23, Level::High, OutputDrive::Standard);
    let mut buzzer = Output::new(p.P0_06, Level::High, OutputDrive::Standard);

    {
        let sda = p.P0_10;
        let scl = p.P0_09;
        let mut config = twim::Config::default();
        let mut i2c = Twim::new(
            p.TWISPI0,
            interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0),
            sda,
            scl,
            config,
        );

        // IMU (not mounted)
        //let mut rx = [0; 1];
        //i2c.write_read(0x68, &[0x75], &mut rx).await.unwrap();
        //info!("rxd: {:02x}", &rx);

        // EEPROM
        //let mut rx = [0; 1];
        //i2c.write_read(0x50, &[0x75], &mut rx).await.unwrap();
        //info!("rxd: {:02x}", &rx);

        // RTC
        //let mut rx = [0; 16];
        //i2c.write_read(0x51, &[0x00, 0x00], &mut rx).await.unwrap();
        //info!("rxd: {:02x}", &rx);

        // SCAN
        //Timer::after(Duration::from_secs(1)).await;
        //for addr in 0..=127 {
        //    if i2c.write(addr, &[0x00]).await.is_ok() {
        //        info!("addr: {:02x}", addr);
        //    }
        //}
        //info!("SCAN DONE");
    }

    let mut rst = Output::new(p.P0_15, Level::High, OutputDrive::Standard);
    let mut irq = Input::new(p.P0_20, Pull::None);

    {
        let miso = p.P0_16;
        let mosi = p.P0_17;
        let sck = p.P0_18;
        let cs = p.P0_19;

        let mut config = Config::default();
        config.frequency = Frequency::M1;
        let spi = Spim::new(p.SPI2, interrupt::take!(SPIM2_SPIS2_SPI2), sck, miso, mosi, config);
        let cs = Output::new(cs, Level::High, OutputDrive::HighDrive);
        let iface = SpiInterface::new(spi, cs);
        let mut fm = Fm175xx::new(iface, rst, irq);

        fm.reset();
        //fm.poweroff();

        fm.lpcd().await;
    }

    rst.set_low();

    loop {
        irq.wait_for_low().await;
        info!("CARD DETECTED");

        rst.set_high();
        delay(640_000); // 10ms

        rst.set_low();
        delay(640_000); // 10ms
    }

    //fm.poweroff();

    pending::<()>().await;

    /*
    loop {
        info!(" ");
        info!(" ");
        fm.init_iso14443a();
        let mut poller = Poller::new(&mut fm);
        let mut card = poller.poll().await;
        let mut isodep = IsoDepA::new(card).await.unwrap();
        let mut rx = [0; 256];
        let tx = [0x90, 0x60, 0x00, 0x00, 0x00];
        let n = isodep.transceive(&tx, &mut rx).await.unwrap();
        info!("rxd: {:02x}", &rx[..n]);

        fm.off();

        Timer::after(Duration::from_secs(1)).await;
    }
     */

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
