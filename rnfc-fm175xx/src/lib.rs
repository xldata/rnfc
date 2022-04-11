#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

// Must go FIRST so that other mods see its macros.
mod fmt;

mod interface;
mod regs;

pub use interface::*;

use core::fmt::Debug;
use core::future::Future;
use cortex_m::asm::delay;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use regs::Regs;
use rnfc_traits::iso14443a_ll::{Reader, TransceiveOptions, TransceiveResult};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Other,
    Timeout,
}

pub struct LpcdConfig {
    pub t1cfg: u8,
    pub t2cfg: u8,
    pub t3cfg: u8,
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
    NpdPin::Error: Debug,
    IrqPin::Error: Debug,
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

    pub async fn lpcd(&mut self) {
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
        });

        self.regs().lpcd_ctrl2().write(|w| {
            w.set_tx2en(true);
            w.set_cwn(true);
            w.set_cwp(0b011);
        });

        self.regs().lpcd_ctrl3().write(|w| w.set_hpden(false));

        let config = LpcdConfig {
            t1cfg: 1,  // 1-15. Tsleep = (t1cfg+2) * 100ms
            t2cfg: 13, // 2-31
            t3cfg: 11, // 2-31
        };

        let (t3clkdiv, adc_shift) = match config.t3cfg {
            16.. => (regs::LpcdT3clkdivk::DIV16, 3),
            8.. => (regs::LpcdT3clkdivk::DIV8, 4),
            0.. => (regs::LpcdT3clkdivk::DIV4, 5),
        };

        let threshold: u8 = 34;

        let adc_range = (config.t3cfg - 1) << adc_shift;
        let adc_center = adc_range / 2;
        let threshold_offs = ((adc_range as u32) * (threshold as u32) / 256) as u8;
        let threshold_min = adc_center.saturating_sub(threshold_offs);
        let threshold_max = adc_center.saturating_add(threshold_offs);

        info!(
            "adc: range={} center={} threshold_offs={} threshold_min={} threshold_max={}",
            adc_range, adc_center, threshold_offs, threshold_min, threshold_max
        );

        self.regs().lpcd_t1cfg().write(|w| {
            w.set_t1cfg(config.t1cfg);
            w.set_t3clkdivk(t3clkdiv);
        });
        self.regs().lpcd_t2cfg().write(|w| w.set_t2cfg(config.t2cfg));
        self.regs().lpcd_t3cfg().write(|w| w.set_t3cfg(config.t3cfg));
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
            self.lpcd_read_adc() < adc_center
        });
        let level = match level {
            Some(x) => x,
            None => panic!("Gain calibration failed."),
        };
        self.regs().lpcd_ctrl4().write_value(levels[level].into());

        // Second, find lowest reference voltage that satisfies "reading < center".
        let reference = binary_search(ADC_REFERENCE_MIN as _, ADC_REFERENCE_MAX as _, |val| {
            self.lpcd_set_adc_config(val as _, 0);
            self.lpcd_read_adc() < adc_center
        });
        let reference = match reference {
            Some(x) => x as u8,
            None => panic!("Reference voltage calibration failed."),
        };
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

    pub fn reset(&mut self) {
        self.npd.set_low().unwrap();
        delay(640_000); // 10ms
        self.npd.set_high().unwrap();

        debug!("softreset");
        self.regs().command().write(|w| w.set_command(regs::CommandVal::SOFTRESET));
        delay(640_000);
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

    pub fn init_iso14443a(&mut self) {
        self.regs().txcontrol().write(|w| {
            w.set_tx1rfen(true);
            w.set_tx2rfen(true);
            w.set_invtx2on(true);
        });

        delay(640_000); // 10ms
        self.regs().txmode().write(|w| {
            w.set_framing(regs::Framing::ISO14443A);
            w.set_speed(regs::Speed::_106KBPS);
        });
        self.regs().rxmode().write(|w| {
            w.set_framing(regs::Framing::ISO14443A);
            w.set_speed(regs::Speed::_106KBPS);
        });
        self.regs().modwidth().write_value(0x26);
        self.regs().gsn().write(|w| {
            w.set_cwgsn(15);
            w.set_modgsn(8);
        });
        self.regs().cwgsp().write(|w| {
            w.set_cwgsp(31);
        });

        self.regs().control().write(|w| {
            w.set_initiator(true);
        });

        self.regs().rfcfg().write(|w| {
            w.set_rxgain(regs::Rxgain::_33DB);
        });
        self.regs().rxtreshold().write(|w| {
            w.set_collevel(4);
            w.set_minlevel(8);
        });
        self.regs().txauto().write(|w| {
            w.set_force100ask(true);
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

impl<I: Interface, NpdPin, IrqPin> Reader for Fm175xx<I, NpdPin, IrqPin>
where
    NpdPin: OutputPin,
    IrqPin: InputPin + Wait,
    NpdPin::Error: Debug,
    <IrqPin as InputPin>::Error: Debug,
    <IrqPin as Wait>::Error: Debug,
{
    type Error = Error;

    type TransceiveFuture<'a> = impl Future<Output = Result<TransceiveResult, Self::Error>>
    where
        Self: 'a;

    fn transceive<'a>(&'a mut self, opts: TransceiveOptions<'a>) -> Self::TransceiveFuture<'a> {
        async move {
            // Disable CRC
            self.regs().txmode().modify(|w| w.set_crcen(opts.crc));
            self.regs().rxmode().modify(|w| w.set_crcen(opts.crc));

            // SEt 1ms timeout
            self.set_timer(opts.timeout_ms);

            // Halt whatever currently running command.
            self.regs().command().write(|w| {
                w.set_command(regs::CommandVal::IDLE);
            });

            self.regs().waterlevel().write(|w| {
                w.set_waterlevel(32);
            });

            // Clear all IRQs
            self.regs().divirq().write_value(0x7f.into());
            self.regs().commirq().write_value(0x7f.into());

            self.clear_fifo();

            // Start trx
            self.regs().command().write(|w| {
                w.set_command(regs::CommandVal::TRANSCEIVE);
            });

            // TODO chunk tx if it's bigger than 64 bytes (the fifo size)
            self.iface.write_fifo(&opts.tx[..((opts.bits + 7) / 8)]);

            self.regs().bitframing().write(|w| {
                w.set_startsend(true);
                w.set_txlastbits((opts.bits % 8) as u8);
            });

            let mut collision = false;

            loop {
                let mut irqs = self.regs().commirq().read();

                if irqs.timeri() {
                    trace!("irq: timeri");
                    return Err(Error::Timeout);
                }

                if irqs.erri() {
                    trace!("irq: ERR");
                    let errs = self.regs().error().read();
                    if errs.collerr() {
                        debug!("err: collision");
                        collision = true;
                        //break;
                    }
                    if errs.bufferovfl() {
                        debug!("err: buffer overflow ");
                        return Err(Error::Other);
                    }
                    if errs.crcerr() {
                        debug!("err: bad CRC");
                        return Err(Error::Other);
                    }
                    //if errs.parityerr() && !collision {
                    //    debug!("err: parity");
                    //    return Err(Error::Other);
                    //}
                    if errs.proterr() {
                        debug!("err: protocol");
                        return Err(Error::Other);
                    }
                    if errs.rferr() {
                        debug!("err: rf");
                        return Err(Error::Other);
                    }
                    if errs.temperr() {
                        debug!("err: temperature");
                        return Err(Error::Other);
                    }
                    if errs.wrerr() {
                        debug!("err: write access error??");
                        return Err(Error::Other);
                    }
                }
                //if irqs.hialerti() {
                //    trace!("irq: hialerti");
                //}
                //if irqs.loalerti() {
                //    trace!("irq: loalerti");
                //}
                if irqs.idlei() {
                    trace!("irq: idle");
                }
                if irqs.rxi() {
                    trace!("irq: rx done");
                    break;
                }
                if irqs.txi() {
                    trace!("irq: tx done");
                }

                irqs.set_set(false);
                self.regs().commirq().write_value(irqs);
            }

            // TODO allow rxing more than 64bytes
            let bytes = self.regs().fifolevel().read().level() as usize;
            if bytes > opts.rx.len() {
                warn!(
                    "rx overflow! received {} bytes but buffer is only {} bytes",
                    bytes,
                    opts.rx.len()
                );
                return Err(Error::Other);
            }

            self.iface.read_fifo(&mut opts.rx[..bytes]);

            let bits = if collision {
                let coll = self.regs().coll().read();
                if coll.collposnotvalid() {
                    warn!("collision position out of range");
                    return Err(Error::Other);
                }

                let mut collpos = coll.collpos() as usize;
                if collpos == 0 {
                    collpos = 32;
                }
                debug!("collision at: collpos={}", collpos);

                // Collision at bit `i` means that bit is not valid, only `0..i-1` are.
                // substract 1 because collpos is 1-based, not 0-based (why??)
                collpos - 1
            } else {
                let mut last_bits = self.regs().control().read().rxbits() as usize;
                if last_bits == 0 {
                    last_bits = 8
                }
                bytes * 8 + last_bits - 8
            };

            Ok(TransceiveResult { bits, collision })
        }
    }
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
