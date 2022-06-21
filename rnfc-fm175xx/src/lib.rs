#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

// Must go FIRST so that other mods see its macros.
mod fmt;

mod interface;
pub mod iso14443a;
mod regs;

use core::convert::Infallible;

use cortex_m::asm::delay;
use embassy::time::{Duration, Timer};
use embedded_hal::digital::blocking::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use regs::Regs;

pub use interface::*;

pub struct WakeupConfig {
    /// sleep time. 1-15. Tsleep = (t1 + 2) * 100ms
    pub t1: u8,
    /// mesaure prepare time? 2-31
    pub t2: u8,
    /// measure time. 2-31
    pub t3: u8,

    // NMOS carrier wave drive strength. 0..=1
    pub n_drive: u8,
    // PMOS carrier wave drive strength. 0..=7
    pub p_drive: u8,
}

const ADC_REFERENCE_MIN: u8 = 0;
const ADC_REFERENCE_MAX: u8 = 0x7F;

pub struct Fm175xx<I: Interface, NpdPin, IrqPin> {
    iface: I,
    npd: NpdPin,
    irq: IrqPin,
}

impl<I: Interface, NpdPin, IrqPin> Fm175xx<I, NpdPin, IrqPin>
where
    NpdPin: OutputPin,
    IrqPin: InputPin + Wait,
{
    pub fn new(iface: I, npd: NpdPin, irq: IrqPin) -> Self {
        Self { iface, npd, irq }
    }

    fn regs(&mut self) -> Regs<I> {
        Regs {
            iface: &mut self.iface,
            addr: 0,
        }
    }

    pub fn poweroff(&mut self) {
        self.regs().command().write(|w| {
            w.set_powerdown(true);
            w.set_rcvoff(true);
        });
    }

    pub async fn lpcd_reset(&mut self) {
        // nRST=0
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false);
            w.set_rstn(true);
        });

        // nRST=1
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true);
            w.set_rstn(true);
        });
    }

    pub async fn wait_for_card(&mut self, config: WakeupConfig) -> Result<(), Infallible> {
        assert!((1..=15).contains(&config.t1));
        assert!((2..=31).contains(&config.t2));
        assert!((2..=31).contains(&config.t3));
        assert!((0..=1).contains(&config.n_drive));
        assert!((0..=7).contains(&config.p_drive));

        self.regs().command().write(|_| {});
        self.regs().commien().write(|w| w.set_irqinv(true));
        self.regs().divien().write(|w| w.set_irqpushpull(true));

        // nRST=0
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false);
            w.set_rstn(true);
        });

        // nRST=1
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true);
            w.set_rstn(true);
        });

        // EN=1
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true);
            w.set_en(true);
        });

        // IE=1
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true);
            w.set_ie(true);
            w.set_sense_1(true);
        });

        self.regs().lpcd_ctrl2().write(|w| {
            w.set_tx2en(true);
            w.set_cwn(config.n_drive == 1);
            w.set_cwp(config.p_drive);
        });

        self.regs().lpcd_ctrl3().write(|w| w.set_hpden(false));

        let (t3clkdiv, adc_shift) = match config.t3 {
            16.. => (regs::LpcdT3clkdivk::DIV16, 3),
            8.. => (regs::LpcdT3clkdivk::DIV8, 4),
            0.. => (regs::LpcdT3clkdivk::DIV4, 5),
        };

        let threshold: u8 = 34;

        let adc_range = (config.t3 - 1) << adc_shift;
        let adc_center = adc_range / 2;
        let threshold_offs = ((adc_range as u32) * (threshold as u32) / 256) as u8;
        let threshold_min = adc_center.saturating_sub(threshold_offs);
        let threshold_max = adc_center.saturating_add(threshold_offs);

        debug!(
            "adc: range={} center={} threshold_offs={} threshold_min={} threshold_max={}",
            adc_range, adc_center, threshold_offs, threshold_min, threshold_max
        );

        self.regs().lpcd_t1cfg().write(|w| {
            w.set_t1cfg(config.t1);
            w.set_t3clkdivk(t3clkdiv);
        });
        self.regs().lpcd_t2cfg().write(|w| w.set_t2cfg(config.t2));
        self.regs().lpcd_t3cfg().write(|w| w.set_t3cfg(config.t3));
        self.regs().lpcd_vmid_bd_cfg().write(|w| w.set_vmid_bd_cfg(8));
        self.regs().lpcd_auto_wup_cfg().write(|w| w.set_en(false));
        self.regs().lpcd_threshold_min_l().write_value(threshold_min & 0x3F);
        self.regs().lpcd_threshold_min_h().write_value(threshold_min >> 6);
        self.regs().lpcd_threshold_max_l().write_value(threshold_max & 0x3F);
        self.regs().lpcd_threshold_max_h().write_value(threshold_max >> 6);

        self.regs().lpcd_misc().write(|w| w.set_calib_vmid_en(true));

        // Calibrate! Note that:
        // - Higher gain -> lower ADC reading
        // - Higher reference voltage -> lower ADC reading

        // First, find lowest gain (multiplier/divider) that satisfies "reading < center".
        self.lpcd_set_adc_config(ADC_REFERENCE_MAX, 0);
        let levels: [u8; 11] = [
            0b000_00, 0b000_10, 0b000_01, 0b000_11, 0b001_11, 0b010_11, 0b011_11, 0b100_11, 0b101_11, 0b110_11, 0b111_11,
        ];
        let level = binary_search(0, levels.len(), |val| {
            self.regs().lpcd_ctrl4().write_value(levels[val].into());
            let meas = self.lpcd_read_adc();
            let res = meas < adc_center;
            debug!("adc search level: {} => {} {}", val, meas, res);
            res
        });
        let level = match level {
            Some(x) => x,
            None => panic!("Gain calibration failed."),
        };
        debug!("adc level {}", level);
        self.regs().lpcd_ctrl4().write_value(levels[level].into());

        // Second, find lowest reference voltage that satisfies "reading < center".
        let reference = binary_search(ADC_REFERENCE_MIN as _, ADC_REFERENCE_MAX as _, |val| {
            self.lpcd_set_adc_config(val as _, 0);
            let meas = self.lpcd_read_adc();
            let res = meas < adc_center;
            debug!("adc search refer: {} => {} {}", val, meas, res);
            res
        });
        let reference = match reference {
            Some(x) => x as u8,
            None => panic!("Reference voltage calibration failed."),
        };
        debug!("adc refer {}", reference);
        self.lpcd_set_adc_config(reference, 0);

        /*
        loop {
            let r = self.lpcd_read_adc();
            info!(" res: {=u8}", r);
            Timer::after(Duration::from_millis(30)).await;
        }
         */

        self.regs().lpcd_misc().write(|w| w.set_calib_vmid_en(false));
        //self.regs().lpcd_auto_wup_cfg().write(|w| {
        //    w.set_en(false);
        //    w.set_time(regs::LpcdAutoWupTime::_1HOUR);
        //});

        self.npd.set_low().unwrap();

        info!("Waiting for irq...");
        self.irq.wait_for_low().await.unwrap();
        info!("Got irq!!");

        Ok(())
    }

    fn lpcd_set_adc_config(&mut self, reference: u8, bias_current: u8) {
        self.regs().lpcd_adc_referece().write_value(reference & 0x3F);
        self.regs().lpcd_bias_current().write(|w| {
            w.set_adc_referece_h((reference >> 6) != 0);
            w.set_bias_current(bias_current);
        });
    }

    fn lpcd_read_adc(&mut self) -> u8 {
        // TODO is this reset needed?

        // nRST = 0
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false);
            w.set_rstn(true);
        });

        // calibra_en = 0
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false);
            w.set_calibra_en(true);
        });

        // nRST = 1
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true);
            w.set_rstn(true);
        });

        // calibra_en = 1
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true);
            w.set_calibra_en(true);
        });

        delay(640_000); // 10ms

        //info!("calib: waiting for irq..");
        while !self.regs().lpcd_irq().read().calib_irq() {}

        // calibra_en = 0
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false);
            w.set_calibra_en(true);
        });

        let h = self.regs().lpcd_adc_result_h().read();
        let l = self.regs().lpcd_adc_result_l().read();

        // TODO is this reset needed?

        // nRST = 0
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(false);
            w.set_rstn(true);
        });

        // nRST = 1
        self.regs().lpcd_ctrl1().write(|w| {
            w.set_bit_ctrl_set(true);
            w.set_rstn(true);
        });

        ((h & 0x3) << 6) | (l & 0x3f)
    }

    pub async fn reset(&mut self) {
        self.npd.set_low().unwrap();
        Timer::after(Duration::from_millis(10)).await;
        self.npd.set_high().unwrap();
        Timer::after(Duration::from_millis(10)).await;

        debug!("softreset");
        self.regs().command().write(|w| w.set_command(regs::CommandVal::SOFTRESET));
        Timer::after(Duration::from_millis(10)).await;
        assert_eq!(self.regs().command().read().0, 0x20);

        let ver = self.regs().version().read();
        debug!("IC version: {:02x}", ver);
    }

    pub fn off(&mut self) {
        self.regs().txcontrol().write(|w| {
            w.set_tx1rfen(false);
            w.set_tx2rfen(false);
        });
    }

    fn clear_fifo(&mut self) {
        self.regs().fifolevel().write(|w| w.set_flushfifo(true));
    }

    fn set_timer(&mut self, ms: u32) {
        let mut prescaler: u32 = 0;
        let mut timereload: u32 = 0;
        while prescaler < 0xfff {
            timereload = (ms * 13560 - 1) / (prescaler * 2 + 1);

            if timereload < 0xffff {
                break;
            }
            prescaler += 1;
        }
        timereload = timereload & 0xFFFF;
        self.regs().tmode().write(|w| {
            w.set_tauto(true);
            w.set_tprescaler_hi((prescaler >> 8) as u8);
        });
        self.regs().tprescaler().write_value(prescaler as u8);
        self.regs().treloadhi().write_value((timereload >> 8) as u8);
        self.regs().treloadlo().write_value(timereload as u8);
    }

    /*
    fn transceive(&mut self, tx: &[u8], rx: &mut [u8], timeout_ms: u32) -> Result<usize, Error> {
        let (len, bits) = self.transceive_raw(tx, rx, timeout_ms, true, 0)?;
        if bits != 0 {
            warn!("incomplete last byte (got {=u8} bits)", bits);
            return Err(Error::Other);
        }
        Ok(len)
    }
     */
}

/// Find lowest value in min..max (min included, max excluded)
/// satisfying `f(val) = true`.
///
/// If `f` returns `false` for all values, returns `None`.
///
/// `f` is assumed to be monotonically increasing.
fn binary_search(mut min: usize, mut max: usize, mut f: impl FnMut(usize) -> bool) -> Option<usize> {
    let orig_max = max;
    while min + 1 < max {
        let m = (min + max) / 2;
        if f(m) {
            max = m
        } else {
            min = m
        }
    }
    if max == orig_max {
        None
    } else {
        Some(max)
    }
}
