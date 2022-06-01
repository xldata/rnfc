use core::future::Future;
use embassy::time::{Duration, Timer};
use embedded_hal::digital::blocking::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use rnfc_traits::iso14443a_ll as ll;

use crate::{regs, Fm175xx, Interface};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Other,
    Timeout,
}

impl ll::Error for Error {
    fn kind(&self) -> ll::ErrorKind {
        match self {
            Error::Timeout => ll::ErrorKind::NoResponse,
            _ => ll::ErrorKind::Other,
        }
    }
}

impl<I: Interface, NpdPin, IrqPin> Fm175xx<I, NpdPin, IrqPin>
where
    NpdPin: OutputPin,
    IrqPin: InputPin + Wait,
{
    pub async fn init_iso14443a(&mut self) {
        self.npd.set_high().unwrap();
        Timer::after(Duration::from_millis(10)).await;

        self.regs().txcontrol().write(|w| {
            w.set_tx1rfen(true);
            w.set_tx2rfen(true);
            w.set_invtx2on(true);
        });

        Timer::after(Duration::from_millis(10)).await;

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
}

impl<I: Interface, NpdPin, IrqPin> ll::Reader for Fm175xx<I, NpdPin, IrqPin>
where
    NpdPin: OutputPin,
    IrqPin: InputPin + Wait,
{
    type Error = Error;

    type TransceiveFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn transceive<'a>(&'a mut self, tx: &'a [u8], rx: &'a mut [u8], opts: ll::Frame) -> Self::TransceiveFuture<'a> {
        async move {
            debug!("TX: {:?} {:02x}", opts, tx);

            let (tx, crc, timeout_ms, lastbits, rxalign) = match opts {
                ll::Frame::Anticoll { bits } => (&tx[..(bits + 7) / 8], false, 1, (bits % 8) as u8, (bits % 8) as u8),
                ll::Frame::ReqA => (&[0x26][..], false, 1, 7, 0),
                ll::Frame::WupA => (&[0x52][..], false, 1, 7, 0),
                ll::Frame::Standard { timeout_ms } => (tx, true, timeout_ms, 0, 0),
            };

            // Disable CRC
            self.regs().txmode().modify(|w| w.set_crcen(crc));
            self.regs().rxmode().modify(|w| w.set_crcen(crc));

            // SEt timeout
            self.set_timer(timeout_ms);

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

            self.regs().coll().write(|w| {
                w.set_valuesaftercoll(!matches!(opts, ll::Frame::Anticoll { .. }));
            });

            // Start trx
            self.regs().command().write(|w| {
                w.set_command(regs::CommandVal::TRANSCEIVE);
            });

            // TODO chunk tx if it's bigger than 64 bytes (the fifo size)
            self.iface.write_fifo(&tx);

            self.regs().bitframing().write(|w| {
                w.set_startsend(true);
                w.set_rxalign(rxalign);
                w.set_txlastbits(lastbits);
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
            if bytes > rx.len() {
                warn!("rx overflow! received {} bytes but buffer is only {} bytes", bytes, rx.len());
                return Err(Error::Other);
            }

            if let ll::Frame::Anticoll { bits } = opts {
                rx.fill(0);
                rx[..bits / 8].copy_from_slice(&tx[..bits / 8]);
                self.iface.read_fifo(&mut rx[bits / 8..][..bytes]);
                if bits % 8 != 0 {
                    let byte_part = tx[bits / 8];
                    let mask = 1u8 << (bits % 8) - 1;
                    rx[bits / 8] |= byte_part & mask;
                }
                for i in 0..(bits + 7) / 8 {
                    rx[i] |= tx[i];
                }

                // Collision at bit `i` means that bit is not valid, only `0..i-1` are.
                // substract 1 because collpos is 1-based, not 0-based (why??)
                let total_bits = if collision {
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
                    bits + collpos - 1
                } else {
                    bits / 8 * 8 + bytes * 8
                };

                debug!("RX: {:02x} bits: {}", rx, total_bits);

                Ok(total_bits)
            } else {
                // TODO: error on collision if not anticollision frame.
                self.iface.read_fifo(&mut rx[..bytes]);
                debug!("RX: {:02x}", &rx[..bytes]);
                Ok(bytes * 8)
            }
        }
    }
}
