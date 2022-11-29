use rnfc_traits::iso14443a::Reader as Iso14443aReader;
use rnfc_traits::iso_dep::Reader as IsoDepReader;

pub const ATS_MAX_LEN: usize = 32; // TODO??

const FSC_MAX: usize = 256;
const FSC_MAX_WITHOUT_CRC: usize = FSC_MAX - 2;

pub struct IsoDepA<T: Iso14443aReader> {
    card: T,

    /// Max frame size we can send to the card, including header and crc.
    /// Ex: if header is 1 byte (no CID/NAD) then max INF field size is FSC-3.
    fsc: usize,

    /// Block count spin bit: 0 or 1
    spinny_bit: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
    Iso14443a(E),
    Protocol,
    TxFrameTooBig,
    RxFrameTooBig,
}

// Divide by 2 so it fits in u8, saving some space
const FS_DIV_2_TABLE: [u8; 9] = [
    16 / 2,
    24 / 2,
    32 / 2,
    40 / 2,
    48 / 2,
    64 / 2,
    96 / 2,
    128 / 2,
    128, // 256 / 2
];

impl<T: Iso14443aReader> IsoDepA<T>
where
    T::Error: crate::fmt::Format,
{
    pub async fn new(mut card: T) -> Result<Self, Error<T::Error>> {
        // RATS
        let req = [0xe0, 0x80];
        let mut res = [0; ATS_MAX_LEN];
        let res_len = match card.transceive(&req, &mut res).await {
            Ok(len) => len,
            Err(e) => {
                warn!("Trx RATS failed: {:?}", e);
                return Err(Error::Iso14443a(e));
            }
        };
        let ats = &res[..res_len];

        if ats.len() < 2 {
            warn!("ATS too short");
            return Err(Error::Protocol);
        }

        let fsci = (ats[1] & 0xF) as usize;
        if fsci >= FS_DIV_2_TABLE.len() {
            warn!("FSCI too high");
            return Err(Error::Protocol);
        }
        let fsc = FS_DIV_2_TABLE[fsci] as usize * 2;
        debug!("fsc = {}", fsc);

        Ok(Self {
            card,
            fsc,
            spinny_bit: 0,
        })
    }

    pub fn inner(&self) -> &T {
        &self.card
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.card
    }

    pub async fn deselect(&mut self) -> Result<(), Error<T::Error>> {
        let tx_buf = [0xC2];
        let mut rx_buf = [0; 1];

        let rx_len = self.card.transceive(&tx_buf, &mut rx_buf).await.map_err(Error::Iso14443a)?;
        if rx_len != 1 || rx_buf != [0xC2] {
            return Err(Error::Protocol);
        }

        Ok(())
    }
}

impl<T: Iso14443aReader> IsoDepReader for IsoDepA<T> {
    type Error = Error<T::Error>;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, Self::Error> {
        let mut tx_buf = [0; FSC_MAX_WITHOUT_CRC];
        let mut rx_buf = [0; FSC_MAX_WITHOUT_CRC];

        if tx.len() + 3 > self.fsc {
            warn!("TX len bigger than FSC: {}+3 > {}", tx.len(), self.fsc);
            return Err(Error::TxFrameTooBig);
        }

        let tx_pcb = 0x02 | self.spinny_bit;

        let mut tx_len = 1 + tx.len();
        tx_buf[0] = tx_pcb;
        tx_buf[1..tx_len].copy_from_slice(tx);

        let rx_len = loop {
            let rx_len = self
                .card
                .transceive(&tx_buf[..tx_len], &mut rx_buf)
                .await
                .map_err(Error::Iso14443a)?;

            if rx_len == 0 {
                warn!("isodep: received zero len data");
                return Err(Error::Protocol);
            }

            // S-block Waiting Time Extension - WTX
            if rx_buf[0] == 0xF2 {
                if rx_len != 2 {
                    warn!("isodep: invalid S(WTX) len {}", rx_len);
                    return Err(Error::Protocol);
                }

                tx_len = 2;
                tx_buf[0] = 0xF2;
                tx_buf[1] = rx_buf[1] & 0x3F;
            } else {
                break rx_len;
            }
        };

        let rx_pcb = rx_buf[0]; // protocol control byte (aka header)

        // TODO this checks the spinny bit is equal, is this guaranteed?
        if rx_pcb != tx_pcb {
            panic!("Receiving chaining, R-blocks or S-blocks is TODO");
        }

        let rx_inf_len = rx_len - 1;
        if rx_inf_len > rx.len() {
            return Err(Error::RxFrameTooBig);
        }

        rx[..rx_inf_len].copy_from_slice(&rx_buf[1..rx_inf_len + 1]);

        // spin the spinny bit
        self.spinny_bit ^= 1;

        Ok(rx_inf_len)
    }
}
