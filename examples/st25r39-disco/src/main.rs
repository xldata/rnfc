#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]

// Must go FIRST
mod fmt;

use core::sync::atomic::{AtomicUsize, Ordering};
use defmt_rtt as _; // global logger
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_stm32::dma::NoDma;
use embassy_stm32::gpio::{Input, Level, Output, Speed};
use embassy_stm32::rcc::{self};
use embassy_stm32::spi::{Config, Phase, Polarity, Spi};
use embassy_stm32::time::Hertz;
use embassy_stm32::Peripherals;
use panic_probe as _;

use rnfc::iso_dep::IsoDepA;
use rnfc::traits::iso_dep::Reader;
use rnfc_st25r39::{SpiInterface, St25r39};

defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        // NOTE(no-CAS) `timestamps` runs with interrupts disabled
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}

fn config() -> embassy_stm32::Config {
    let mut cfg = embassy_stm32::Config::default();
    cfg.rcc.mux = rcc::ClockSrc::PLL(
        rcc::PLLSource::HSI16,
        rcc::PLLClkDiv::Div2,
        rcc::PLLSrcDiv::Div1,
        rcc::PLLMul::Mul8,
        Some(rcc::PLLClkDiv::Div2),
    );
    cfg
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    //let mut led = Output::new(p.PC4, Level::High, Speed::Low);

    let mut config = Config::default();
    config.mode.polarity = Polarity::IdleLow;
    config.mode.phase = Phase::CaptureOnSecondTransition;
    let spi = Spi::new(p.SPI1, p.PA5, p.PA7, p.PE14, NoDma, NoDma, Hertz(1_000_000), config);

    let cs = Output::new(p.PA4, Level::High, Speed::VeryHigh);

    let iface = SpiInterface::new(spi, cs);
    let mut st = St25r39::new(iface).await;

    let _irq = Input::new(p.PE15, embassy_stm32::gpio::Pull::None);

    /*
    let config = WakeupConfig {
        period: WakeupPeriod::Ms500,
        capacitive: None,
        inductive_amplitude: None,
        inductive_phase: Some(WakeupMethodConfig {
            delta: 3,
            reference: WakeupReference::Automatic,
        }),
    };

    st.mode_wakeup(config).await;

    info!("Waiting for pin irq");
    while irq.is_low() {}
    info!("yay");
     */

    let tag = loop {
        match st.select_iso14443a().await {
            Ok(c) => break c,
            Err(e) => warn!("select failed: {:?}", e),
        }
        Timer::after(Duration::from_millis(100)).await;
    };

    let mut tag = IsoDepA::new(tag).await.unwrap();

    let mut rx = [0; 256];
    let tx = [0x90, 0x60, 0x00, 0x00, 0x00];
    let n = tag.transceive(&tx, &mut rx).await.unwrap();
    info!("rxd: {:02x}", &rx[..n]);

    info!("DONE");

    cortex_m::asm::bkpt();
}
