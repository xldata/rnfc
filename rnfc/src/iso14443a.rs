use core::fmt::Debug;
use core::future::Future;

use heapless::Vec;
use rnfc_traits::iso14443a::{Reader, UID_MAX_LEN};
use rnfc_traits::iso14443a_ll::{Reader as LLReader, TransceiveOptions};

struct UidPart {
    uid: u32,
    bits: u8,
}

fn keep_low_bits(val: u64, bits: u32) -> u64 {
    let shift = 64u32.saturating_sub(bits);
    (val << shift) >> shift
}

pub struct Poller<T: LLReader> {
    reader: T,
}

impl<T: LLReader> Poller<T>
where
    T::Error: Debug,
{
    pub fn new(reader: T) -> Self {
        Self { reader }
    }

    async fn transceive_wupa(&mut self) -> [u8; 2] {
        let tx = [0x52];
        let mut rx = [0; 2];
        let opts = TransceiveOptions {
            tx: &tx,
            rx: &mut rx,
            crc: false,
            bits: 7,
            timeout_ms: 1,
        };
        info!("tx WUPA");
        let res = self.reader.transceive(opts).await.unwrap();
        info!("rxd: {:02x} bits={} coll={}", &rx, res.bits, res.collision);
        assert_eq!(res.bits, 16);
        rx
    }

    #[allow(unused)]
    async fn transceive_reqa(&mut self) -> [u8; 2] {
        let tx = [0x25];
        let mut rx = [0; 2];
        let opts = TransceiveOptions {
            tx: &tx,
            rx: &mut rx,
            crc: false,
            bits: 7,
            timeout_ms: 1,
        };
        info!("tx REQA");
        let res = self.reader.transceive(opts).await.unwrap();
        info!("rxd: {:02x} bits={} coll={}", &rx, res.bits, res.collision);
        assert_eq!(res.bits, 16);
        rx
    }

    async fn transceive_anticoll(&mut self, cl: u8, uid_part: UidPart) -> UidPart {
        let bits = 16 + uid_part.bits;

        // Build frame
        let mut tx = [0; 6];
        tx[0] = 0x93 + cl * 2;
        tx[1] = ((bits / 8) << 4) | (bits % 8);
        tx[2..].copy_from_slice(&uid_part.uid.to_le_bytes());

        info!("tx ANTICOLL: {:02x} bits={}", &tx, bits);

        let mut rx = [0; 8];
        let opts = TransceiveOptions {
            tx: &tx,
            rx: &mut rx,
            crc: false,
            bits: bits as usize,
            timeout_ms: 5,
        };
        let res = self.reader.transceive(opts).await.unwrap();
        info!("rxd: {:02x} bits={} coll={}", &rx, res.bits, res.collision);

        // If first bit is a collision, we haven't learned any new on the UID.
        // Treat it as a communication error to prevent infinite loops.
        if res.bits == 0 {
            panic!("anticoll: got zero new bits");
        }

        let new_part = keep_low_bits(u64::from_le_bytes(rx), res.bits as _);
        let combined = uid_part.uid as u64 | (new_part << uid_part.bits);
        let combined_bits = uid_part.bits + res.bits as u8;

        info!("combined: {:02x} bits={}", combined.to_le_bytes(), combined_bits);

        // If we don't have the full 32bit UID yet, return
        if combined_bits < 32 {
            return UidPart {
                uid: combined as _,
                bits: combined_bits,
            };
        }

        // We got a complete UID. We should have exactly 40 bits. It's a protocol error otherwise:
        // If we have 32..39, it means a collision occured during the BCC bit which should be impossible.
        // If we have more than 40, it means the card is responding with too many bits.
        if combined_bits != 40 {
            panic!("anticoll: got bad combined bits {}", combined_bits);
        }

        let bcc = combined_bits as u32;
        let bcc = bcc ^ bcc >> 16;
        let bcc = bcc ^ bcc >> 8;

        if bcc as u8 != combined_bits >> 32 as u8 {
            panic!("bad BCC");
        }

        // Return the complete UID!
        UidPart {
            uid: combined as _,
            bits: 32,
        }
    }

    async fn transceive_select(&mut self, cl: u8, uid: [u8; 4]) -> u8 {
        // Build frame
        let mut tx = [0; 7];
        tx[0] = 0x93 + cl * 2;
        tx[1] = 0x70; // 7 bytes, 0, bits
        tx[2..6].copy_from_slice(&uid);
        tx[6] = uid[0] ^ uid[1] ^ uid[2] ^ uid[3];
        let mut rx = [0; 1];
        let opts = TransceiveOptions {
            tx: &tx,
            rx: &mut rx,
            crc: true,
            bits: 7 * 8,
            timeout_ms: 1,
        };
        let res = self.reader.transceive(opts).await.unwrap();
        info!("rxd: {:02x} bits={} coll={}", &rx, res.bits, res.collision);
        assert_eq!(res.bits, 8);
        rx[0]
    }

    pub async fn poll(&mut self) -> Card<'_, T> {
        let atqa = self.transceive_wupa().await;

        let mut uid: Vec<u8, UID_MAX_LEN> = Vec::new();
        let mut sak = 0;

        for cl in 0..4 {
            if cl == 3 {
                warn!("too many cascade levels");
                panic!("klfjlas");
            }

            let mut uid_part = UidPart { uid: 0, bits: 0 };
            loop {
                uid_part = self.transceive_anticoll(cl, uid_part).await;
                if uid_part.bits == 32 {
                    break;
                }

                uid_part.bits += 1;
                //self.transceive_select().await;
            }

            let uid_part = uid_part.uid.to_le_bytes();
            sak = self.transceive_select(cl, uid_part).await;

            if uid_part[0] == 0x88 {
                uid.extend_from_slice(&uid_part[1..]).unwrap();
            } else {
                uid.extend_from_slice(&uid_part).unwrap();
                break;
            }
        }

        Card {
            reader: &mut self.reader,
            uid,
            atqa,
            sak,
        }
    }
}

pub struct Card<'d, T: LLReader> {
    reader: &'d mut T,

    uid: Vec<u8, UID_MAX_LEN>,
    atqa: [u8; 2],
    sak: u8,
}

impl<'d, T: LLReader> Reader for Card<'d, T>
where
    T::Error: Debug,
{
    type Error = ();

    type TransceiveFuture<'a> = impl Future<Output = Result<usize, Self::Error>> + 'a where Self: 'a;

    fn transceive<'a>(&'a mut self, tx: &'a [u8], rx: &'a mut [u8]) -> Self::TransceiveFuture<'a> {
        async move {
            let opts = TransceiveOptions {
                tx,
                rx,
                crc: true,
                bits: tx.len() * 8,
                timeout_ms: 100, // TODO unhardcode
            };
            let res = self.reader.transceive(opts).await.unwrap();
            if res.bits % 8 != 0 {
                panic!("last byte was not complete!");
            }
            Ok(res.bits / 8)
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
