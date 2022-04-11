use core::future::Future;
use embassy::time::{with_timeout, Duration, Timer, TICKS_PER_SECOND};
use heapless::Vec;
use rnfc_traits::iso14443a::Reader;

use crate::*;

// NFC-A minimum FDT(listen) = ((n * 128 + (84)) / fc) with n_min = 9      Digital 1.1  6.10.1
//                           = (1236)/fc
// Relax with 3etu: (3*128)/fc as with multiple NFC-A cards, response may take longer (JCOP cards)
//                           = (1236 + 384)/fc = 1620 / fc
const NFCA_FDTMIN: Duration = duration_from_1fc(1620);

const fn duration_from_1fc(n: u64) -> Duration {
    // + 1 to round up
    Duration::from_ticks(n * TICKS_PER_SECOND / 13_560_000 + 1)
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TxFrame<'a> {
    ReqA,
    WupA,
    Anticoll(&'a [u8]),
    Standard(&'a [u8]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Timeout,

    Framing,
    FramingLastByteIncomplete,
    FramingLastByteMissingParity,

    Crc,
    Collision,
    Parity,
    ResponseTooShort,
    ResponseTooLong,

    FifoOverflow,
    FifoUnderflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SelectError {
    FieldCollision,
    Timeout,
    Protocol,
}

impl<I: Interface> St25r39<I> {
    pub async fn select_iso14443a<'a>(&'a mut self) -> Result<Tag<'a, I>, SelectError> {
        self.mode_on().await;
        match self.field_on().await {
            Ok(()) => {}
            Err(FieldOnError::FieldCollision) => {
                self.mode_off().await;
                return Err(SelectError::FieldCollision);
            }
        }

        // Field on guard time
        Timer::after(Duration::from_millis(5)).await;

        for _try in 0..3 {
            if let Ok(x) = self.select_iso14443a_inner().await {
                return Ok(Tag {
                    device: self,
                    uid: x.uid,
                    atqa: x.atqa,
                    sak: x.sak,
                });
            }
        }
        self.mode_off().await;
        Err(SelectError::Timeout)
    }

    async fn select_iso14443a_inner(&mut self) -> Result<TagInfo, SelectError> {
        let mut atqa = [0; 2];
        match self.transceive(TxFrame::ReqA, &mut atqa, NFCA_FDTMIN).await {
            Ok(_) => {}
            Err(e) => {
                debug!("Tx REQA failed: {:?}", e);
                return Err(SelectError::Protocol);
            }
        }

        let mut id: Vec<u8, UID_MAX_LEN> = Vec::new();
        let mut sak = 0;

        for lv in 0..4 {
            if lv == 3 {
                warn!("too many cascade levels");
                return Err(SelectError::Protocol);
            }

            let id_part = self.transceive_anticoll(lv).await.map_err(|e| {
                warn!("select: trx anticollision failed: {:?}", e);
                SelectError::Protocol
            })?;
            sak = self.transceive_select(lv, id_part).await.map_err(|e| {
                warn!("select: trx select failed: {:?}", e);
                SelectError::Protocol
            })?;

            if id_part[0] == 0x88 {
                id.extend_from_slice(&id_part[1..]).unwrap();
            } else {
                id.extend_from_slice(&id_part).unwrap();
                break;
            }
        }

        Ok(TagInfo { uid: id, atqa, sak })
    }

    async fn transceive_anticoll(&mut self, cascade_lv: u8) -> Result<[u8; 4], Error> {
        let req = [0x93 + cascade_lv * 2, 0x20];
        let mut res = [0; 5];
        let res_len = match self.transceive(TxFrame::Anticoll(&req), &mut res, NFCA_FDTMIN).await {
            Ok(len) => len,
            Err(e) => {
                warn!("Tx anticoll failed: {:?}", e);
                return Err(Error::Collision);
            }
        };

        if res_len != 5 {
            warn!("bad anticoll len {}", res_len);
            return Err(Error::Collision);
        }

        if res[0] ^ res[1] ^ res[2] ^ res[3] != res[4] {
            warn!("bad anticoll BCC");
            return Err(Error::Collision);
        }

        Ok([res[0], res[1], res[2], res[3]])
    }

    async fn transceive_select(&mut self, cascade_lv: u8, id: [u8; 4]) -> Result<u8, Error> {
        let bcc = id[0] ^ id[1] ^ id[2] ^ id[3];
        let req = [0x93 + cascade_lv * 2, 0x70, id[0], id[1], id[2], id[3], bcc];
        let mut res = [0; 1];
        let fwt = Duration::from_millis(100); // TODO: unhardcode.
        let res_len = match self.transceive(TxFrame::Standard(&req), &mut res, fwt).await {
            Ok(len) => len,
            Err(e) => {
                warn!("Tx select failed: {:?}", e);
                return Err(Error::Collision);
            }
        };

        // todo validate res
        let _ = res_len;

        Ok(res[0])
    }

    async fn transceive(&mut self, tx: TxFrame<'_>, rx: &mut [u8], fwt: Duration) -> Result<usize, Error> {
        debug!("TX: {:02x}", tx);

        let (raw, cmd, data) = match tx {
            TxFrame::ReqA => (true, Command::TransmitReqa, None),
            TxFrame::WupA => (true, Command::TransmitWupa, None),
            TxFrame::Anticoll(data) => {
                self.regs().corr_conf1().write_value(0x11.into());
                (true, Command::TransmitWithoutCrc, Some(data))
            }
            TxFrame::Standard(data) => (false, Command::TransmitWithCrc, Some(data)),
        };

        self.cmd(Command::Stop);
        self.cmd(Command::ResetRxgain);

        self.regs().aux().write(|w| {
            w.set_no_crc_rx(raw);
        });

        if let Some(data) = data {
            let data_bits = data.len() * 8;
            self.regs().num_tx_bytes2().write_value((data_bits as u8).into());
            self.regs().num_tx_bytes1().write_value((data_bits >> 8) as u8);

            if !data.is_empty() {
                self.iface.write_fifo(data);
            }
        }

        self.irqs = 0; // stop already clears all irqs
        self.cmd(cmd);

        // Wait for tx ended
        self.irq_wait(Interrupt::Txe).await;

        // Wait for RX started, with max FWT.
        with_timeout(
            fwt,
            // Wait for rx started
            self.irq_wait(Interrupt::Rxs),
        )
        .await
        .map_err(|_| Error::Timeout)?;

        // Wait for rx ended or error
        // The timeout should never hit, it's just for safety.
        let res = with_timeout(Duration::from_millis(500), async {
            loop {
                if self.irq(Interrupt::Err1) {
                    return Err(Error::Framing);
                }
                if self.irq(Interrupt::Par) {
                    return Err(Error::Parity);
                }
                if self.irq(Interrupt::Crc) {
                    return Err(Error::Crc);
                }
                if self.irq(Interrupt::Col) {
                    return Err(Error::Collision);
                }

                if self.irq(Interrupt::Rxe) {
                    break;
                }

                yield_now().await;
                self.irq_update();
            }
            Ok(())
        })
        .await;

        match res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(Error::Timeout),
        }

        // If we're here, RX ended without error.

        let stat = self.regs().fifo_status2().read();
        if stat.fifo_ovr() {
            return Err(Error::FifoOverflow);
        }
        if stat.fifo_unf() {
            return Err(Error::FifoUnderflow);
        }
        if stat.fifo_lb() != 0 {
            debug!("fifo_lb = {=u8}", stat.fifo_lb());
            return Err(Error::FramingLastByteIncomplete);
        }
        if stat.np_lb() {
            return Err(Error::FramingLastByteMissingParity);
        }

        let mut rx_bytes = self.regs().fifo_status1().read() as usize;
        rx_bytes |= (stat.fifo_b() as usize) << 8;

        // Remove received CRC
        if !raw {
            if rx_bytes < 2 {
                return Err(Error::ResponseTooShort);
            }
            rx_bytes -= 2;
        }

        if rx.len() < rx_bytes {
            return Err(Error::ResponseTooLong);
        }

        self.iface.read_fifo(&mut rx[..rx_bytes]);
        debug!("RX: {=[u8]:02x}", &mut rx[..rx_bytes]);

        Ok(rx_bytes)
    }
}

pub const UID_MAX_LEN: usize = 10;

pub struct Tag<'d, I: Interface> {
    pub(crate) device: &'d mut St25r39<I>,

    pub(crate) uid: Vec<u8, UID_MAX_LEN>,
    pub(crate) atqa: [u8; 2],
    pub(crate) sak: u8,
}

pub struct TagInfo {
    pub(crate) uid: Vec<u8, UID_MAX_LEN>,
    pub(crate) atqa: [u8; 2],
    pub(crate) sak: u8,
}

impl<'d, I: Interface> Drop for Tag<'d, I> {
    fn drop(&mut self) {
        self.device.mode_off_inner()
    }
}

impl<'d, I: Interface> Reader for Tag<'d, I> {
    type Error = Error;

    type TransceiveFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a where Self: 'a ;

    fn transceive<'a>(&'a mut self, tx: &'a [u8], rx: &'a mut [u8]) -> Self::TransceiveFuture<'a> {
        async move {
            let fwt = Duration::from_millis(100); // TODO: unhardcode.
            self.device.transceive(TxFrame::Standard(tx), rx, fwt).await.map_err(|e| {
                warn!("Trx failed: {:?}", e);
                e
            })
        }
    }

    fn uid(&self) -> &[u8] {
        &self.uid
    }

    fn atqa(&self) -> [u8; 2] {
        self.atqa
    }

    fn sak(&self) -> u8 {
        self.sak
    }
}
