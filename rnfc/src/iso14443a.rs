use heapless::Vec;
use rnfc_traits::iso14443a::{Reader, UID_MAX_LEN};
use rnfc_traits::iso14443a_ll as ll;
use rnfc_traits::iso14443a_ll::{Frame, Reader as LLReader};

use crate::fmt::Bytes;

macro_rules! retry {
    ($tries:literal, $expr:expr) => {{
        let mut tries = $tries;
        loop {
            let r = $expr;
            if let Ok(r) = r {
                break Ok(r);
            }

            tries -= 1;

            if tries == 0 {
                break r;
            }
        }
    }};
}

pub struct Poller<T: LLReader> {
    reader: T,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<T> {
    Lower(T),
    Protocol,
}

impl<T: ll::Error> Error<T> {
    fn is_soft(&self) -> bool {
        match self {
            Self::Lower(l) => l.kind() == ll::ErrorKind::NoResponse,
            Self::Protocol => true,
        }
    }
}

impl<T: LLReader> Poller<T> {
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    async fn transceive_wupa(&mut self) -> Result<[u8; 2], Error<T::Error>> {
        let mut rx = [0; 2];
        let bits = self
            .reader
            .transceive(&[], &mut rx, Frame::WupA)
            .await
            .map_err(Error::Lower)?;
        if bits != 16 {
            debug!("WUPA response wrong length: {} bits", bits);
            return Err(Error::Protocol);
        }
        Ok(rx)
    }

    #[allow(unused)]
    async fn transceive_reqa(&mut self) -> Result<[u8; 2], Error<T::Error>> {
        let mut rx = [0; 2];
        let bits = self
            .reader
            .transceive(&[], &mut rx, Frame::ReqA)
            .await
            .map_err(Error::Lower)?;
        if bits != 16 {
            debug!("REQA response wrong length: {} bits", bits);
            return Err(Error::Protocol);
        }
        Ok(rx)
    }

    async fn transceive_anticoll(&mut self, cl: u8, uid: &mut [u8; 4], uid_bits: usize) -> Result<usize, Error<T::Error>> {
        let bits = 16 + uid_bits as u8;

        // Build frame
        let mut tx = [0; 6];
        tx[0] = 0x93 + cl * 2;
        tx[1] = ((bits / 8) << 4) | (bits % 8);
        tx[2..].copy_from_slice(uid);

        let mut rx = [0; 8];
        let opts = Frame::Anticoll { bits: bits as _ };
        let got_bits = self.reader.transceive(&tx, &mut rx, opts).await.map_err(Error::Lower)?;

        // If first bit is a collision, we haven't learned any new on the UID.
        // Treat it as a communication error to prevent infinite loops.
        if got_bits as u8 == bits {
            debug!("anticoll: got zero new bits");
            return Err(Error::Protocol);
        }
        if got_bits < 16 {
            debug!("collision too early?");
            return Err(Error::Protocol);
        }

        let new_uid_bits = got_bits - 16;

        uid.copy_from_slice(&rx[2..6]);

        // If we don't have the full 32bit UID yet, return
        if new_uid_bits < 32 {
            return Ok(new_uid_bits);
        }

        // We got a complete UID. We should have exactly 40 bits. It's a protocol error otherwise:
        // If we have 32..39, it means a collision occured during the BCC bit which should be impossible.
        // If we have more than 40, it means the card is responding with too many bits.
        if new_uid_bits != 40 {
            debug!("anticoll: got bad new_uid_bits {}", new_uid_bits);
            return Err(Error::Protocol);
        }

        let bcc = uid[0] ^ uid[1] ^ uid[2] ^ uid[3];
        if bcc as u8 != rx[6] {
            debug!("bad BCC");
            return Err(Error::Protocol);
        }

        // Return the complete UID!
        Ok(32)
    }

    async fn transceive_select(&mut self, cl: u8, uid: [u8; 4]) -> Result<u8, Error<T::Error>> {
        // Build frame
        let mut tx = [0; 7];
        tx[0] = 0x93 + cl * 2;
        tx[1] = 0x70; // 7 bytes, 0, bits
        tx[2..6].copy_from_slice(&uid);
        tx[6] = uid[0] ^ uid[1] ^ uid[2] ^ uid[3];
        let mut rx = [0; 1];
        let opts = Frame::Standard { timeout_ms: 1 };
        let bits = self.reader.transceive(&tx, &mut rx, opts).await.map_err(Error::Lower)?;
        if bits != 8 {
            debug!("SELECT response wrong length: {} bits", bits);
            return Err(Error::Protocol);
        }
        Ok(rx[0])
    }

    async fn transceive_hlta(&mut self) -> Result<(), Error<T::Error>> {
        let tx = [0x50, 0x00];
        let mut rx = [0; 1];
        let opts = Frame::Standard { timeout_ms: 1 };
        let _ = self.reader.transceive(&tx, &mut rx, opts).await.map_err(Error::Lower)?;
        Ok(())
    }

    pub async fn select_any(&mut self) -> Result<Card<'_, T>, Error<T::Error>> {
        let atqa = retry!(4, self.transceive_wupa().await)?;

        let mut uid: Vec<u8, UID_MAX_LEN> = Vec::new();
        let mut sak = 0;

        for cl in 0..4 {
            if cl == 3 {
                debug!("too many cascade levels");
                return Err(Error::Protocol);
            }

            let mut uid_part = [0; 4];
            let mut uid_bits = 0;
            loop {
                uid_bits = retry!(4, self.transceive_anticoll(cl, &mut uid_part, uid_bits).await)?;
                if uid_bits == 32 {
                    break;
                }
                uid_bits += 1;
            }

            sak = retry!(4, self.transceive_select(cl, uid_part).await)?;

            if uid_part[0] == 0x88 {
                uid.extend_from_slice(&uid_part[1..]).unwrap();
            } else {
                uid.extend_from_slice(&uid_part).unwrap();
                break;
            }
        }

        debug!("Got card! uid={} atqa={} sak={:02}", Bytes(&uid), Bytes(&atqa), sak);

        Ok(Card {
            reader: &mut self.reader,
            uid,
            atqa,
            sak,
        })
    }

    pub async fn select_by_id(&mut self, uid: &[u8]) -> Result<Card<'_, T>, Error<T::Error>> {
        let atqa = retry!(4, self.transceive_wupa().await)?;

        let mut sak = 0;

        let cln = match uid.len() {
            4 => 1,
            7 => 2,
            10 => 3,
            x => {
                debug!("Invalid UID length {}", x);
                return Err(Error::Protocol);
            }
        };

        for cl in 0..cln {
            let uid_part = if cl == cln - 1 {
                [uid[cl * 3], uid[cl * 3 + 1], uid[cl * 3 + 2], uid[cl * 3 + 3]]
            } else {
                [0x88, uid[cl * 3], uid[cl * 3 + 1], uid[cl * 3 + 2]]
            };

            sak = retry!(4, self.transceive_select(cl as u8, uid_part).await)?;
        }

        debug!("Got card! uid={} atqa={} sak={:02}", Bytes(&uid), Bytes(&atqa), sak);

        Ok(Card {
            reader: &mut self.reader,
            uid: Vec::from_slice(uid).unwrap(),
            atqa,
            sak,
        })
    }

    /// Search for all cards in the field, and return a list of their IDs.
    /// You can connect to one with [`Self::select_by_id`].
    pub async fn search<const N: usize>(&mut self) -> Result<Vec<Vec<u8, UID_MAX_LEN>, N>, Error<T::Error>> {
        let mut res = Vec::new();

        'out: for _ in 0..(N * 4) {
            let atqa = match retry!(4, self.transceive_reqa().await) {
                Ok(x) => x,
                Err(e) if e.is_soft() => break,
                Err(e) => return Err(e),
            };

            let mut uid: Vec<u8, UID_MAX_LEN> = Vec::new();
            let mut sak = 0;

            for cl in 0..4 {
                if cl == 3 {
                    debug!("too many cascade levels");
                    return Err(Error::Protocol);
                }

                let mut uid_part = [0; 4];
                let mut uid_bits = 0;
                loop {
                    uid_bits = match retry!(4, self.transceive_anticoll(cl, &mut uid_part, uid_bits).await) {
                        Ok(x) => x,
                        Err(e) if e.is_soft() => break 'out,
                        Err(e) => return Err(e),
                    };
                    if uid_bits == 32 {
                        break;
                    }
                    uid_bits += 1;
                }

                sak = match retry!(4, self.transceive_select(cl, uid_part).await) {
                    Ok(x) => x,
                    Err(e) if e.is_soft() => break 'out,
                    Err(e) => return Err(e),
                };

                if uid_part[0] == 0x88 {
                    uid.extend_from_slice(&uid_part[1..]).unwrap();
                } else {
                    uid.extend_from_slice(&uid_part).unwrap();
                    break;
                }
            }

            debug!("Got card! uid={} atqa={} sak={:02}", Bytes(&uid), Bytes(&atqa), sak);
            let _ = self.transceive_hlta().await;

            if !res.contains(&uid) {
                res.push(uid).unwrap();
                if res.is_full() {
                    break;
                }
            }
        }

        Ok(res)
    }
}

pub struct Card<'d, T: LLReader> {
    reader: &'d mut T,

    uid: Vec<u8, UID_MAX_LEN>,
    atqa: [u8; 2],
    sak: u8,
}

impl<'d, T: LLReader + 'd> Reader for Card<'d, T> {
    type Error = T::Error;

    async fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, Self::Error> {
        let opts = Frame::Standard {
            timeout_ms: 100, // TODO unhardcode
        };
        let res = self.reader.transceive(tx, rx, opts).await?;
        if res % 8 != 0 {
            panic!("last byte was not complete!");
        }
        Ok(res / 8)
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
