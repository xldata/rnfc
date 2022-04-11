#![allow(unused)]

use core::marker::PhantomData;

use super::Interface;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RW;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct R;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct W;

mod sealed {
    use super::*;
    pub trait Access {}
    impl Access for R {}
    impl Access for W {}
    impl Access for RW {}
}

pub trait Access: sealed::Access + Copy {}
impl Access for R {}
impl Access for W {}
impl Access for RW {}

pub trait Read: Access {}
impl Read for RW {}
impl Read for R {}

pub trait Write: Access {}
impl Write for RW {}
impl Write for W {}

pub struct Reg<'a, I: Interface, T: Copy, A: Access> {
    addr: usize,
    iface: &'a mut I,
    phantom: PhantomData<(A, &'a mut T)>,
}

impl<'a, I: Interface, T: Copy + Into<u8> + From<u8>, A: Access> Reg<'a, I, T, A> {
    pub fn new(iface: &'a mut I, addr: usize) -> Self {
        Self {
            iface,
            addr,
            phantom: PhantomData,
        }
    }

    pub fn read(&mut self) -> T
    where
        A: Read,
    {
        self.iface.read_reg(self.addr).into()
    }

    pub fn write_value(&mut self, val: T)
    where
        A: Write,
    {
        self.iface.write_reg(self.addr, val.into())
    }

    pub fn modify<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R
    where
        A: Read + Write,
    {
        let mut val = self.read();
        let res = f(&mut val);
        self.write_value(val);
        res
    }

    pub fn write<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R
    where
        A: Write,
        T: Default,
    {
        let mut val = Default::default();
        let res = f(&mut val);
        self.write_value(val);
        res
    }
}

// ==========================================================
// ==========================================================
// ==========================================================
pub struct Regs<'a, I> {
    pub iface: &'a mut I,
    pub addr: usize,
}
impl<'a, I: Interface> Regs<'a, I> {
    #[doc = "Page register in page 0"]
    #[inline(always)]
    pub fn page0(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 0usize) }
    }
    #[doc = "Contains Command bits, PowerDown bit and bit to switch receiver off."]
    #[inline(always)]
    pub fn command(&mut self) -> self::Reg<'_, I, Command, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 1usize) }
    }
    #[doc = "Contains Communication interrupt enable bits andbit for Interrupt inversion."]
    #[inline(always)]
    pub fn commien(&mut self) -> self::Reg<'_, I, Commien, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 2usize) }
    }
    #[doc = "Contains RfOn, RfOff, CRC and Mode Interrupt enable and bit to switch Interrupt pin to PushPull mode."]
    #[inline(always)]
    pub fn divien(&mut self) -> self::Reg<'_, I, Divien, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 3usize) }
    }
    #[doc = "Contains Communication interrupt request bits."]
    #[inline(always)]
    pub fn commirq(&mut self) -> self::Reg<'_, I, Commirq, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 4usize) }
    }
    #[doc = "Contains RfOn, RfOff, CRC and Mode Interrupt request."]
    #[inline(always)]
    pub fn divirq(&mut self) -> self::Reg<'_, I, Divirq, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 5usize) }
    }
    #[doc = "Contains Protocol, Parity, CRC, Collision, Buffer overflow, Temperature and RF error flags."]
    #[inline(always)]
    pub fn error(&mut self) -> self::Reg<'_, I, Error, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 6usize) }
    }
    #[doc = "Contains status information about Lo- and HiAlert, RF-field on, Timer, Interrupt request and CRC status."]
    #[inline(always)]
    pub fn status1(&mut self) -> self::Reg<'_, I, Status1, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 7usize) }
    }
    #[doc = "Contains information about internal states (Modemstate),Mifare states and possibility to switch Temperature sensor off."]
    #[inline(always)]
    pub fn status2(&mut self) -> self::Reg<'_, I, Status2, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 8usize) }
    }
    #[doc = "Gives access to FIFO. Writing to register increments theFIFO level (register 0x0A), reading decrements it."]
    #[inline(always)]
    pub fn fifodata(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 9usize) }
    }
    #[doc = "Contains the actual level of the FIFO."]
    #[inline(always)]
    pub fn fifolevel(&mut self) -> self::Reg<'_, I, Fifolevel, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 10usize) }
    }
    #[doc = "Contains the Waterlevel value for the FIFO"]
    #[inline(always)]
    pub fn waterlevel(&mut self) -> self::Reg<'_, I, Waterlevel, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 11usize) }
    }
    #[doc = "Contains information about last received bits, Initiator mode bit, bit to copy NFCID to FIFO and to Start and stopthe Timer unit."]
    #[inline(always)]
    pub fn control(&mut self) -> self::Reg<'_, I, Control, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 12usize) }
    }
    #[doc = "Contains information of last bits to send, to align received bits in FIFO and activate sending in Transceive"]
    #[inline(always)]
    pub fn bitframing(&mut self) -> self::Reg<'_, I, Bitframing, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 13usize) }
    }
    #[doc = "Contains all necessary bits for Collission handling"]
    #[inline(always)]
    pub fn coll(&mut self) -> self::Reg<'_, I, Coll, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 14usize) }
    }
    #[doc = "Currently not used."]
    #[inline(always)]
    pub fn rfu0f(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 15usize) }
    }
    #[doc = "Page register in page 1"]
    #[inline(always)]
    pub fn page1(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 16usize) }
    }
    #[doc = "Contains bits for auto wait on Rf, to detect SYNC byte in NFC mode and MSB first for CRC calculation"]
    #[inline(always)]
    pub fn mode(&mut self) -> self::Reg<'_, I, Mode, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 17usize) }
    }
    #[doc = "Contains Transmit Framing, Speed, CRC enable, bit for inverse mode and TXMix bit."]
    #[inline(always)]
    pub fn txmode(&mut self) -> self::Reg<'_, I, Txmode, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 18usize) }
    }
    #[doc = "Contains Transmit Framing, Speed, CRC enable, bit for multiple receive and to filter errors."]
    #[inline(always)]
    pub fn rxmode(&mut self) -> self::Reg<'_, I, Rxmode, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 19usize) }
    }
    #[doc = "Contains bits to activate and configure Tx1 and Tx2 and bit to activate 100% modulation."]
    #[inline(always)]
    pub fn txcontrol(&mut self) -> self::Reg<'_, I, Txcontrol, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 20usize) }
    }
    #[doc = "Contains bits to automatically switch on/off the Rf and to do the collission avoidance and the initial rf-on."]
    #[inline(always)]
    pub fn txauto(&mut self) -> self::Reg<'_, I, Txauto, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 21usize) }
    }
    #[doc = "Contains SigoutSel, DriverSel and LoadModSel bits."]
    #[inline(always)]
    pub fn txsel(&mut self) -> self::Reg<'_, I, Txsel, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 22usize) }
    }
    #[doc = "Contains UartSel and RxWait bits."]
    #[inline(always)]
    pub fn rxsel(&mut self) -> self::Reg<'_, I, Rxsel, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 23usize) }
    }
    #[doc = "Contains MinLevel and CollLevel for detection."]
    #[inline(always)]
    pub fn rxtreshold(&mut self) -> self::Reg<'_, I, Rxtreshold, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 24usize) }
    }
    #[doc = "Contains bits for time constants, hysteresis and IQ demodulator settings."]
    #[inline(always)]
    pub fn demod(&mut self) -> self::Reg<'_, I, Demod, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 25usize) }
    }
    #[doc = "Contains bits for minimum FeliCa length received and for FeliCa syncronisation length."]
    #[inline(always)]
    pub fn felicanfc(&mut self) -> self::Reg<'_, I, Felicanfc, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 26usize) }
    }
    #[doc = "Contains bits for maximum FeliCa length received."]
    #[inline(always)]
    pub fn felicanfc2(&mut self) -> self::Reg<'_, I, Felicanfc2, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 27usize) }
    }
    #[doc = "Contains Miller settings, TxWait settings and MIFARE halted mode bit."]
    #[inline(always)]
    pub fn mifare(&mut self) -> self::Reg<'_, I, Mifare, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 28usize) }
    }
    #[doc = "Currently not used."]
    #[inline(always)]
    pub fn manualrcv(&mut self) -> self::Reg<'_, I, Manualrcv, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 29usize) }
    }
    #[doc = "Currently not used."]
    #[inline(always)]
    pub fn rfu1e(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 30usize) }
    }
    #[doc = "Contains speed settings for serila interface."]
    #[inline(always)]
    pub fn serialspeed(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 31usize) }
    }
    #[doc = "Page register in page 2"]
    #[inline(always)]
    pub fn page2(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 32usize) }
    }
    #[doc = "Contains MSByte of CRC Result."]
    #[inline(always)]
    pub fn crcresult1(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 33usize) }
    }
    #[doc = "Contains LSByte of CRC Result."]
    #[inline(always)]
    pub fn crcresult2(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 34usize) }
    }
    #[doc = "Contains the conductance and the modulation settings for the N-MOS transistor only for load modulation (See difference to JREG_GSN!)."]
    #[inline(always)]
    pub fn gsnloadmod(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 35usize) }
    }
    #[doc = "Contains modulation width setting."]
    #[inline(always)]
    pub fn modwidth(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 36usize) }
    }
    #[doc = "Contains TxBitphase settings and receive clock change."]
    #[inline(always)]
    pub fn txbitphase(&mut self) -> self::Reg<'_, I, Txbitphase, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 37usize) }
    }
    #[doc = "Contains sensitivity of Rf Level detector, the receiver gain factor and the RfLevelAmp."]
    #[inline(always)]
    pub fn rfcfg(&mut self) -> self::Reg<'_, I, Rfcfg, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 38usize) }
    }
    #[doc = "Contains the conductance and the modulation settings for the N-MOS transistor during active modulation (no load modulation setting!)."]
    #[inline(always)]
    pub fn gsn(&mut self) -> self::Reg<'_, I, Gsn, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 39usize) }
    }
    #[doc = "Contains the conductance for the P-Mos transistor."]
    #[inline(always)]
    pub fn cwgsp(&mut self) -> self::Reg<'_, I, Cwgsp, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 40usize) }
    }
    #[doc = "Contains the modulation index for the PMos transistor."]
    #[inline(always)]
    pub fn modgsp(&mut self) -> self::Reg<'_, I, Modgsp, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 41usize) }
    }
    #[doc = "Contains all settings for the timer and the highest 4 bits of the prescaler."]
    #[inline(always)]
    pub fn tmode(&mut self) -> self::Reg<'_, I, Tmode, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 42usize) }
    }
    #[doc = "Contais the lowest byte of the prescaler."]
    #[inline(always)]
    pub fn tprescaler(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 43usize) }
    }
    #[doc = "Contains the high byte of the reload value."]
    #[inline(always)]
    pub fn treloadhi(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 44usize) }
    }
    #[doc = "Contains the low byte of the reload value."]
    #[inline(always)]
    pub fn treloadlo(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 45usize) }
    }
    #[doc = "Contains the high byte of the counter value."]
    #[inline(always)]
    pub fn tcountervalhi(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 46usize) }
    }
    #[doc = "Contains the low byte of the counter value."]
    #[inline(always)]
    pub fn tcountervallo(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 47usize) }
    }
    #[doc = "Page register in page 3"]
    #[inline(always)]
    pub fn page3(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 48usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn testsel1(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 49usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn testsel2(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 50usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn testpinen(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 51usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn testpinvalue(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 52usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn testbus(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 53usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn autotest(&mut self) -> self::Reg<'_, I, Autotest, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 54usize) }
    }
    #[doc = "Contains the product number and the version ."]
    #[inline(always)]
    pub fn version(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 55usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn analogtest(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 56usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn testdac1(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 57usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn testdac2(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 58usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn testadc(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 59usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn analoguetest1(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 60usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn analoguetest0(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 61usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn analoguetpd_a(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 62usize) }
    }
    #[doc = "Test register"]
    #[inline(always)]
    pub fn analoguetpd_b(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 63usize) }
    }
    #[doc = "Lpcd Ctrl register1"]
    #[inline(always)]
    pub fn lpcd_ctrl1(&mut self) -> self::Reg<'_, I, LpcdCtrl1, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 65usize) }
    }
    #[doc = "Lpcd Ctrl register2"]
    #[inline(always)]
    pub fn lpcd_ctrl2(&mut self) -> self::Reg<'_, I, LpcdCtrl2, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 66usize) }
    }
    #[doc = "Lpcd Ctrl register3"]
    #[inline(always)]
    pub fn lpcd_ctrl3(&mut self) -> self::Reg<'_, I, LpcdCtrl3, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 67usize) }
    }
    #[doc = "Lpcd Ctrl register4"]
    #[inline(always)]
    pub fn lpcd_ctrl4(&mut self) -> self::Reg<'_, I, LpcdCtrl4, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 68usize) }
    }
    #[doc = "Lpcd bias current register"]
    #[inline(always)]
    pub fn lpcd_bias_current(&mut self) -> self::Reg<'_, I, LpcdBiasCurrent, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 69usize) }
    }
    #[doc = "Lpcd adc reference register"]
    #[inline(always)]
    pub fn lpcd_adc_referece(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 70usize) }
    }
    #[doc = "T1Cfg\\[3:0\\] register"]
    #[inline(always)]
    pub fn lpcd_t1cfg(&mut self) -> self::Reg<'_, I, LpcdT1cfg, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 71usize) }
    }
    #[doc = "T2Cfg\\[4:0\\] register"]
    #[inline(always)]
    pub fn lpcd_t2cfg(&mut self) -> self::Reg<'_, I, LpcdT2cfg, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 72usize) }
    }
    #[doc = "T2Cfg\\[4:0\\] register"]
    #[inline(always)]
    pub fn lpcd_t3cfg(&mut self) -> self::Reg<'_, I, LpcdT3cfg, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 73usize) }
    }
    #[doc = "VmidBdCfg\\[4:0\\] register"]
    #[inline(always)]
    pub fn lpcd_vmid_bd_cfg(&mut self) -> self::Reg<'_, I, LpcdVmidBdCfg, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 74usize) }
    }
    #[doc = "Auto_Wup_Cfg register"]
    #[inline(always)]
    pub fn lpcd_auto_wup_cfg(&mut self) -> self::Reg<'_, I, LpcdAutoWupCfg, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 75usize) }
    }
    #[doc = "ADCResult\\[5:0\\] Register"]
    #[inline(always)]
    pub fn lpcd_adc_result_l(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 76usize) }
    }
    #[doc = "ADCResult\\[7:6\\] Register"]
    #[inline(always)]
    pub fn lpcd_adc_result_h(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 77usize) }
    }
    #[doc = "LpcdThreshold_L\\[5:0\\] Register"]
    #[inline(always)]
    pub fn lpcd_threshold_min_l(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 78usize) }
    }
    #[doc = "LpcdThreshold_L\\[7:6\\] Register"]
    #[inline(always)]
    pub fn lpcd_threshold_min_h(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 79usize) }
    }
    #[doc = "LpcdThreshold_H\\[5:0\\] Register"]
    #[inline(always)]
    pub fn lpcd_threshold_max_l(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 80usize) }
    }
    #[doc = "LpcdThreshold_H\\[7:6\\] Register"]
    #[inline(always)]
    pub fn lpcd_threshold_max_h(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 81usize) }
    }
    #[doc = "LpcdStatus Register"]
    #[inline(always)]
    pub fn lpcd_irq(&mut self) -> self::Reg<'_, I, LpcdIrq, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 82usize) }
    }
    #[doc = "Aux1 select Register"]
    #[inline(always)]
    pub fn lpcd_rft1(&mut self) -> self::Reg<'_, I, LpcdRft1, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 83usize) }
    }
    #[doc = "Aux2 select Register"]
    #[inline(always)]
    pub fn lpcd_rft2(&mut self) -> self::Reg<'_, I, LpcdRft2, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 84usize) }
    }
    #[doc = "LPCD test1 Register"]
    #[inline(always)]
    pub fn lpcd_rft3(&mut self) -> self::Reg<'_, I, LpcdRft3, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 85usize) }
    }
    #[doc = "LPCD test2 Register"]
    #[inline(always)]
    pub fn lpcd_rft4(&mut self) -> self::Reg<'_, I, LpcdRft4, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 86usize) }
    }
    #[doc = "lp_clk_cnt\\[5:0\\] Register"]
    #[inline(always)]
    pub fn lp_clk_cnt1(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 87usize) }
    }
    #[doc = "lp_clk_cnt\\[7:6\\] Register"]
    #[inline(always)]
    pub fn lp_clk_cnt2(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 88usize) }
    }
    #[doc = "VersionReg2\\[1:0\\] Register"]
    #[inline(always)]
    pub fn versionreg2(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 89usize) }
    }
    #[doc = "Irq bak Register"]
    #[inline(always)]
    pub fn irq_bak(&mut self) -> self::Reg<'_, I, IrqBak, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 90usize) }
    }
    #[doc = "LPCD TEST3 Register"]
    #[inline(always)]
    pub fn lpcd_rft5(&mut self) -> self::Reg<'_, I, LpcdRft5, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 91usize) }
    }
    #[doc = "LPCD Misc Register"]
    #[inline(always)]
    pub fn lpcd_misc(&mut self) -> self::Reg<'_, I, LpcdMisc, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 92usize) }
    }
    #[doc = "Low Votage Detect register"]
    #[inline(always)]
    pub fn lvd_ctrl(&mut self) -> self::Reg<'_, I, u8, self::RW> {
        unsafe { self::Reg::new(self.iface, self.addr + 93usize) }
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Demod(pub u8);
impl Demod {
    #[doc = ""]
    #[inline(always)]
    pub const fn tausync(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_tausync(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val as u8) & 0x03) << 0usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn taurcv(&self) -> u8 {
        let val = (self.0 >> 2usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_taurcv(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 2usize)) | (((val as u8) & 0x03) << 2usize);
    }
    #[doc = "If set to 1 and the lower bit of AddIQ is set to 0, the receiving is fixed to I channel. If set to 1 and the lower bit of AddIQ is set to 1, the receiving is fixed to Q channel."]
    #[inline(always)]
    pub const fn fixiq(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "If set to 1 and the lower bit of AddIQ is set to 0, the receiving is fixed to I channel. If set to 1 and the lower bit of AddIQ is set to 1, the receiving is fixed to Q channel."]
    #[inline(always)]
    pub fn set_fixiq(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn addiq(&self) -> u8 {
        let val = (self.0 >> 6usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_addiq(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 6usize)) | (((val as u8) & 0x03) << 6usize);
    }
}
impl Default for Demod {
    #[inline(always)]
    fn default() -> Demod {
        Demod(0)
    }
}
impl From<u8> for Demod {
    fn from(val: u8) -> Demod {
        Demod(val)
    }
}
impl From<Demod> for u8 {
    fn from(val: Demod) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdBiasCurrent(pub u8);
impl LpcdBiasCurrent {
    #[doc = "Bias current"]
    #[inline(always)]
    pub const fn bias_current(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x07;
        val as u8
    }
    #[doc = "Bias current"]
    #[inline(always)]
    pub fn set_bias_current(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val as u8) & 0x07) << 0usize);
    }
    #[doc = "ADC reference level bit 6"]
    #[inline(always)]
    pub const fn adc_referece_h(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "ADC reference level bit 6"]
    #[inline(always)]
    pub fn set_adc_referece_h(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
}
impl Default for LpcdBiasCurrent {
    #[inline(always)]
    fn default() -> LpcdBiasCurrent {
        LpcdBiasCurrent(0)
    }
}
impl From<u8> for LpcdBiasCurrent {
    fn from(val: u8) -> LpcdBiasCurrent {
        LpcdBiasCurrent(val)
    }
}
impl From<LpcdBiasCurrent> for u8 {
    fn from(val: LpcdBiasCurrent) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdT3cfg(pub u8);
impl LpcdT3cfg {
    #[doc = "T3_time = (T3Cfg-1)*4.7us. Valid range 2-0x1F"]
    #[inline(always)]
    pub const fn t3cfg(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x1f;
        val as u8
    }
    #[doc = "T3_time = (T3Cfg-1)*4.7us. Valid range 2-0x1F"]
    #[inline(always)]
    pub fn set_t3cfg(&mut self, val: u8) {
        self.0 = (self.0 & !(0x1f << 0usize)) | (((val as u8) & 0x1f) << 0usize);
    }
}
impl Default for LpcdT3cfg {
    #[inline(always)]
    fn default() -> LpcdT3cfg {
        LpcdT3cfg(0)
    }
}
impl From<u8> for LpcdT3cfg {
    fn from(val: u8) -> LpcdT3cfg {
        LpcdT3cfg(val)
    }
}
impl From<LpcdT3cfg> for u8 {
    fn from(val: LpcdT3cfg) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdCtrl4(pub u8);
impl LpcdCtrl4 {
    #[doc = "Attenuation factor"]
    #[inline(always)]
    pub const fn attenuation(&self) -> LpcdAttenuation {
        let val = (self.0 >> 0usize) & 0x03;
        LpcdAttenuation(val as u8)
    }
    #[doc = "Attenuation factor"]
    #[inline(always)]
    pub fn set_attenuation(&mut self, val: LpcdAttenuation) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val.0 as u8) & 0x03) << 0usize);
    }
    #[doc = "Amplifier multiplier"]
    #[inline(always)]
    pub const fn gain(&self) -> LpcdGain {
        let val = (self.0 >> 2usize) & 0x07;
        LpcdGain(val as u8)
    }
    #[doc = "Amplifier multiplier"]
    #[inline(always)]
    pub fn set_gain(&mut self, val: LpcdGain) {
        self.0 = (self.0 & !(0x07 << 2usize)) | (((val.0 as u8) & 0x07) << 2usize);
    }
}
impl Default for LpcdCtrl4 {
    #[inline(always)]
    fn default() -> LpcdCtrl4 {
        LpcdCtrl4(0)
    }
}
impl From<u8> for LpcdCtrl4 {
    fn from(val: u8) -> LpcdCtrl4 {
        LpcdCtrl4(val)
    }
}
impl From<LpcdCtrl4> for u8 {
    fn from(val: LpcdCtrl4) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Status2(pub u8);
impl Status2 {
    #[doc = "reader status Crypto is on."]
    #[inline(always)]
    pub const fn crypto1on(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "reader status Crypto is on."]
    #[inline(always)]
    pub fn set_crypto1on(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "card status Mifare selected."]
    #[inline(always)]
    pub const fn mfselected(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "card status Mifare selected."]
    #[inline(always)]
    pub fn set_mfselected(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Bit position to forece High speed mode for I2C Interface."]
    #[inline(always)]
    pub const fn i2cforcehs(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "Bit position to forece High speed mode for I2C Interface."]
    #[inline(always)]
    pub fn set_i2cforcehs(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    #[doc = "Bit position to switch Temperture sensors on/off."]
    #[inline(always)]
    pub const fn tempsensoff(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Bit position to switch Temperture sensors on/off."]
    #[inline(always)]
    pub fn set_tempsensoff(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Status2 {
    #[inline(always)]
    fn default() -> Status2 {
        Status2(0)
    }
}
impl From<u8> for Status2 {
    fn from(val: u8) -> Status2 {
        Status2(val)
    }
}
impl From<Status2> for u8 {
    fn from(val: Status2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Waterlevel(pub u8);
impl Waterlevel {
    #[doc = ""]
    #[inline(always)]
    pub const fn waterlevel(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x3f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_waterlevel(&mut self, val: u8) {
        self.0 = (self.0 & !(0x3f << 0usize)) | (((val as u8) & 0x3f) << 0usize);
    }
}
impl Default for Waterlevel {
    #[inline(always)]
    fn default() -> Waterlevel {
        Waterlevel(0)
    }
}
impl From<u8> for Waterlevel {
    fn from(val: u8) -> Waterlevel {
        Waterlevel(val)
    }
}
impl From<Waterlevel> for u8 {
    fn from(val: Waterlevel) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdIrq(pub u8);
impl LpcdIrq {
    #[doc = "irq of card in"]
    #[inline(always)]
    pub const fn card_in_irq(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "irq of card in"]
    #[inline(always)]
    pub fn set_card_in_irq(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "irq of LPCD 23 end"]
    #[inline(always)]
    pub const fn lpcd23_irq(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "irq of LPCD 23 end"]
    #[inline(always)]
    pub fn set_lpcd23_irq(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "irq of calib end"]
    #[inline(always)]
    pub const fn calib_irq(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "irq of calib end"]
    #[inline(always)]
    pub fn set_calib_irq(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "irq of lp osc 10K ok"]
    #[inline(always)]
    pub const fn lp10k_testok_irq(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "irq of lp osc 10K ok"]
    #[inline(always)]
    pub fn set_lp10k_testok_irq(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "irq of Auto wake up"]
    #[inline(always)]
    pub const fn auto_wup_irq(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "irq of Auto wake up"]
    #[inline(always)]
    pub fn set_auto_wup_irq(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
}
impl Default for LpcdIrq {
    #[inline(always)]
    fn default() -> LpcdIrq {
        LpcdIrq(0)
    }
}
impl From<u8> for LpcdIrq {
    fn from(val: u8) -> LpcdIrq {
        LpcdIrq(val)
    }
}
impl From<LpcdIrq> for u8 {
    fn from(val: LpcdIrq) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Gsn(pub u8);
impl Gsn {
    #[doc = "Conductance of theoutput for periods of modulation"]
    #[inline(always)]
    pub const fn modgsn(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    #[doc = "Conductance of theoutput for periods of modulation"]
    #[inline(always)]
    pub fn set_modgsn(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    #[doc = "Conductance of theoutput for periods of no modulation"]
    #[inline(always)]
    pub const fn cwgsn(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    #[doc = "Conductance of theoutput for periods of no modulation"]
    #[inline(always)]
    pub fn set_cwgsn(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for Gsn {
    #[inline(always)]
    fn default() -> Gsn {
        Gsn(0)
    }
}
impl From<u8> for Gsn {
    fn from(val: u8) -> Gsn {
        Gsn(val)
    }
}
impl From<Gsn> for u8 {
    fn from(val: Gsn) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Error(pub u8);
impl Error {
    #[doc = "Protocol Error."]
    #[inline(always)]
    pub const fn proterr(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "Protocol Error."]
    #[inline(always)]
    pub fn set_proterr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "Parity Error."]
    #[inline(always)]
    pub const fn parityerr(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "Parity Error."]
    #[inline(always)]
    pub fn set_parityerr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "CRC Error."]
    #[inline(always)]
    pub const fn crcerr(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "CRC Error."]
    #[inline(always)]
    pub fn set_crcerr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "Collision Error."]
    #[inline(always)]
    pub const fn collerr(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Collision Error."]
    #[inline(always)]
    pub fn set_collerr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "Buffer Overflow Error."]
    #[inline(always)]
    pub const fn bufferovfl(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Buffer Overflow Error."]
    #[inline(always)]
    pub fn set_bufferovfl(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "RF Error."]
    #[inline(always)]
    pub const fn rferr(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "RF Error."]
    #[inline(always)]
    pub fn set_rferr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = "Temerature Error."]
    #[inline(always)]
    pub const fn temperr(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "Temerature Error."]
    #[inline(always)]
    pub fn set_temperr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    #[doc = "Write Access Error."]
    #[inline(always)]
    pub const fn wrerr(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "Write Access Error."]
    #[inline(always)]
    pub fn set_wrerr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
}
impl Default for Error {
    #[inline(always)]
    fn default() -> Error {
        Error(0)
    }
}
impl From<u8> for Error {
    fn from(val: u8) -> Error {
        Error(val)
    }
}
impl From<Error> for u8 {
    fn from(val: Error) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Felicanfc2(pub u8);
impl Felicanfc2 {
    #[doc = ""]
    #[inline(always)]
    pub const fn felicalen(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x3f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_felicalen(&mut self, val: u8) {
        self.0 = (self.0 & !(0x3f << 0usize)) | (((val as u8) & 0x3f) << 0usize);
    }
    #[doc = "If this bit is set to one, the response time to the polling command is half as normal."]
    #[inline(always)]
    pub const fn fasttimeslot(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "If this bit is set to one, the response time to the polling command is half as normal."]
    #[inline(always)]
    pub fn set_fasttimeslot(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn felicasynclen(&self) -> u8 {
        let val = (self.0 >> 6usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_felicasynclen(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 6usize)) | (((val as u8) & 0x03) << 6usize);
    }
    #[doc = "If this bit is set to one, only passive communication modes are possible. In any other case the AutoColl Statemachine does not exit."]
    #[inline(always)]
    pub const fn waitforselected(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "If this bit is set to one, only passive communication modes are possible. In any other case the AutoColl Statemachine does not exit."]
    #[inline(always)]
    pub fn set_waitforselected(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Felicanfc2 {
    #[inline(always)]
    fn default() -> Felicanfc2 {
        Felicanfc2(0)
    }
}
impl From<u8> for Felicanfc2 {
    fn from(val: u8) -> Felicanfc2 {
        Felicanfc2(val)
    }
}
impl From<Felicanfc2> for u8 {
    fn from(val: Felicanfc2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdVmidBdCfg(pub u8);
impl LpcdVmidBdCfg {
    #[doc = "Configure the Vmid establishment time. Configuration Value = T2Cfg Configuration Value - Settling time required for Vmid. \\[Note: It is recommended to use the setting of the reference code, and it is not recommended for the user to modify it\\]"]
    #[inline(always)]
    pub const fn vmid_bd_cfg(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x1f;
        val as u8
    }
    #[doc = "Configure the Vmid establishment time. Configuration Value = T2Cfg Configuration Value - Settling time required for Vmid. \\[Note: It is recommended to use the setting of the reference code, and it is not recommended for the user to modify it\\]"]
    #[inline(always)]
    pub fn set_vmid_bd_cfg(&mut self, val: u8) {
        self.0 = (self.0 & !(0x1f << 0usize)) | (((val as u8) & 0x1f) << 0usize);
    }
}
impl Default for LpcdVmidBdCfg {
    #[inline(always)]
    fn default() -> LpcdVmidBdCfg {
        LpcdVmidBdCfg(0)
    }
}
impl From<u8> for LpcdVmidBdCfg {
    fn from(val: u8) -> LpcdVmidBdCfg {
        LpcdVmidBdCfg(val)
    }
}
impl From<LpcdVmidBdCfg> for u8 {
    fn from(val: LpcdVmidBdCfg) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Control(pub u8);
impl Control {
    #[doc = ""]
    #[inline(always)]
    pub const fn rxbits(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x07;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_rxbits(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val as u8) & 0x07) << 0usize);
    }
    #[doc = "Sets Initiator mode."]
    #[inline(always)]
    pub const fn initiator(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Sets Initiator mode."]
    #[inline(always)]
    pub fn set_initiator(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Copies internal stored NFCID3 to actual position of FIFO."]
    #[inline(always)]
    pub const fn wrnfcidtofifo(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Copies internal stored NFCID3 to actual position of FIFO."]
    #[inline(always)]
    pub fn set_wrnfcidtofifo(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = "Starts timer if set to 1."]
    #[inline(always)]
    pub const fn tstartnow(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "Starts timer if set to 1."]
    #[inline(always)]
    pub fn set_tstartnow(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    #[doc = "Stops timer if set to 1."]
    #[inline(always)]
    pub const fn tstopnow(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Stops timer if set to 1."]
    #[inline(always)]
    pub fn set_tstopnow(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Control {
    #[inline(always)]
    fn default() -> Control {
        Control(0)
    }
}
impl From<u8> for Control {
    fn from(val: u8) -> Control {
        Control(val)
    }
}
impl From<Control> for u8 {
    fn from(val: Control) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Mode(pub u8);
impl Mode {
    #[doc = ""]
    #[inline(always)]
    pub const fn crcpreset(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_crcpreset(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val as u8) & 0x03) << 0usize);
    }
    #[doc = "Deactivates the ModeDetector during AutoAnticoll command. The settings of the register are valid only."]
    #[inline(always)]
    pub const fn modedetoff(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "Deactivates the ModeDetector during AutoAnticoll command. The settings of the register are valid only."]
    #[inline(always)]
    pub fn set_modedetoff(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "Inverts polarity of SiginActIrq, if bit is set to 1 IRQ occures when Sigin line is 0."]
    #[inline(always)]
    pub const fn polsigin(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Inverts polarity of SiginActIrq, if bit is set to 1 IRQ occures when Sigin line is 0."]
    #[inline(always)]
    pub fn set_polsigin(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "Rx waits until Rf is enabled until receive is startet, else receive is started immideately."]
    #[inline(always)]
    pub const fn rxwaitrf(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Rx waits until Rf is enabled until receive is startet, else receive is started immideately."]
    #[inline(always)]
    pub fn set_rxwaitrf(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Tx waits until Rf is enabled until transmit is startet, else transmit is started immideately."]
    #[inline(always)]
    pub const fn txwaitrf(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Tx waits until Rf is enabled until transmit is startet, else transmit is started immideately."]
    #[inline(always)]
    pub fn set_txwaitrf(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = "Activate automatic sync detection for NFC 106kbps."]
    #[inline(always)]
    pub const fn detectsync(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "Activate automatic sync detection for NFC 106kbps."]
    #[inline(always)]
    pub fn set_detectsync(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    #[doc = "Sets CRC coprocessor with MSB first."]
    #[inline(always)]
    pub const fn msbfirst(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Sets CRC coprocessor with MSB first."]
    #[inline(always)]
    pub fn set_msbfirst(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Mode {
    #[inline(always)]
    fn default() -> Mode {
        Mode(0)
    }
}
impl From<u8> for Mode {
    fn from(val: u8) -> Mode {
        Mode(val)
    }
}
impl From<Mode> for u8 {
    fn from(val: Mode) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Felicanfc(pub u8);
impl Felicanfc {
    #[doc = ""]
    #[inline(always)]
    pub const fn felicalen(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x3f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_felicalen(&mut self, val: u8) {
        self.0 = (self.0 & !(0x3f << 0usize)) | (((val as u8) & 0x3f) << 0usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn felicasynclen(&self) -> u8 {
        let val = (self.0 >> 6usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_felicasynclen(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 6usize)) | (((val as u8) & 0x03) << 6usize);
    }
}
impl Default for Felicanfc {
    #[inline(always)]
    fn default() -> Felicanfc {
        Felicanfc(0)
    }
}
impl From<u8> for Felicanfc {
    fn from(val: u8) -> Felicanfc {
        Felicanfc(val)
    }
}
impl From<Felicanfc> for u8 {
    fn from(val: Felicanfc) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdT2cfg(pub u8);
impl LpcdT2cfg {
    #[doc = "T2_time = (T2Cfg+2)*100us. Valid range 2-0x1F"]
    #[inline(always)]
    pub const fn t2cfg(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x1f;
        val as u8
    }
    #[doc = "T2_time = (T2Cfg+2)*100us. Valid range 2-0x1F"]
    #[inline(always)]
    pub fn set_t2cfg(&mut self, val: u8) {
        self.0 = (self.0 & !(0x1f << 0usize)) | (((val as u8) & 0x1f) << 0usize);
    }
}
impl Default for LpcdT2cfg {
    #[inline(always)]
    fn default() -> LpcdT2cfg {
        LpcdT2cfg(0)
    }
}
impl From<u8> for LpcdT2cfg {
    fn from(val: u8) -> LpcdT2cfg {
        LpcdT2cfg(val)
    }
}
impl From<LpcdT2cfg> for u8 {
    fn from(val: LpcdT2cfg) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Rxmode(pub u8);
impl Rxmode {
    #[doc = ""]
    #[inline(always)]
    pub const fn framing(&self) -> Framing {
        let val = (self.0 >> 0usize) & 0x03;
        Framing(val as u8)
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_framing(&mut self, val: Framing) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val.0 as u8) & 0x03) << 0usize);
    }
    #[doc = "Activates reception mode for multiple responses."]
    #[inline(always)]
    pub const fn rxmultiple(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "Activates reception mode for multiple responses."]
    #[inline(always)]
    pub fn set_rxmultiple(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "If 1, receiver does not receive less than 4 bits."]
    #[inline(always)]
    pub const fn rxnoerr(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "If 1, receiver does not receive less than 4 bits."]
    #[inline(always)]
    pub fn set_rxnoerr(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[inline(always)]
    pub const fn speed(&self) -> Speed {
        let val = (self.0 >> 4usize) & 0x07;
        Speed(val as u8)
    }
    #[inline(always)]
    pub fn set_speed(&mut self, val: Speed) {
        self.0 = (self.0 & !(0x07 << 4usize)) | (((val.0 as u8) & 0x07) << 4usize);
    }
    #[doc = "Activates transmit or receive CRC."]
    #[inline(always)]
    pub const fn crcen(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Activates transmit or receive CRC."]
    #[inline(always)]
    pub fn set_crcen(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Rxmode {
    #[inline(always)]
    fn default() -> Rxmode {
        Rxmode(0)
    }
}
impl From<u8> for Rxmode {
    fn from(val: u8) -> Rxmode {
        Rxmode(val)
    }
}
impl From<Rxmode> for u8 {
    fn from(val: Rxmode) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdCtrl3(pub u8);
impl LpcdCtrl3 {
    #[doc = "Low power mode control. Set to 0: When pin NPD=0, and LPCDEn=0 (default value), the chip enters DPD mode. Set to 1: When pin NPD=0, and LPCDEn=0 (default value), the chip enters HPD mode. \\[Note: To enable LPCD function, set as: HPDEn=0, LPCDEn=1\\]"]
    #[inline(always)]
    pub const fn hpden(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Low power mode control. Set to 0: When pin NPD=0, and LPCDEn=0 (default value), the chip enters DPD mode. Set to 1: When pin NPD=0, and LPCDEn=0 (default value), the chip enters HPD mode. \\[Note: To enable LPCD function, set as: HPDEn=0, LPCDEn=1\\]"]
    #[inline(always)]
    pub fn set_hpden(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
}
impl Default for LpcdCtrl3 {
    #[inline(always)]
    fn default() -> LpcdCtrl3 {
        LpcdCtrl3(0)
    }
}
impl From<u8> for LpcdCtrl3 {
    fn from(val: u8) -> LpcdCtrl3 {
        LpcdCtrl3(val)
    }
}
impl From<LpcdCtrl3> for u8 {
    fn from(val: LpcdCtrl3) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdRft1(pub u8);
impl LpcdRft1 {
    #[doc = "vdem of lpcd"]
    #[inline(always)]
    pub const fn aux1_vdem_lpcd(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "vdem of lpcd"]
    #[inline(always)]
    pub fn set_aux1_vdem_lpcd(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "voltage of charecap"]
    #[inline(always)]
    pub const fn aux1_vp_lpcd(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "voltage of charecap"]
    #[inline(always)]
    pub fn set_aux1_vp_lpcd(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
}
impl Default for LpcdRft1 {
    #[inline(always)]
    fn default() -> LpcdRft1 {
        LpcdRft1(0)
    }
}
impl From<u8> for LpcdRft1 {
    fn from(val: u8) -> LpcdRft1 {
        LpcdRft1(val)
    }
}
impl From<LpcdRft1> for u8 {
    fn from(val: LpcdRft1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdT1cfg(pub u8);
impl LpcdT1cfg {
    #[doc = "T1_time = (T1Cfg+2)*100ms. Valid range 1-0xF"]
    #[inline(always)]
    pub const fn t1cfg(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    #[doc = "T1_time = (T1Cfg+2)*100ms. Valid range 1-0xF"]
    #[inline(always)]
    pub fn set_t1cfg(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    #[doc = "Frequency division ratio setting of working clock in T3 stage"]
    #[inline(always)]
    pub const fn t3clkdivk(&self) -> LpcdT3clkdivk {
        let val = (self.0 >> 4usize) & 0x03;
        LpcdT3clkdivk(val as u8)
    }
    #[doc = "Frequency division ratio setting of working clock in T3 stage"]
    #[inline(always)]
    pub fn set_t3clkdivk(&mut self, val: LpcdT3clkdivk) {
        self.0 = (self.0 & !(0x03 << 4usize)) | (((val.0 as u8) & 0x03) << 4usize);
    }
}
impl Default for LpcdT1cfg {
    #[inline(always)]
    fn default() -> LpcdT1cfg {
        LpcdT1cfg(0)
    }
}
impl From<u8> for LpcdT1cfg {
    fn from(val: u8) -> LpcdT1cfg {
        LpcdT1cfg(val)
    }
}
impl From<LpcdT1cfg> for u8 {
    fn from(val: LpcdT1cfg) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdAutoWupCfg(pub u8);
impl LpcdAutoWupCfg {
    #[doc = "To configure automatic wake-up time:"]
    #[inline(always)]
    pub const fn time(&self) -> LpcdAutoWupTime {
        let val = (self.0 >> 0usize) & 0x07;
        LpcdAutoWupTime(val as u8)
    }
    #[doc = "To configure automatic wake-up time:"]
    #[inline(always)]
    pub fn set_time(&mut self, val: LpcdAutoWupTime) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val.0 as u8) & 0x07) << 0usize);
    }
    #[doc = "Set to 1, the chip automatically exits from LPCD mode according to the set AutoWupCfg value, At the same time, the interrupt flag is given."]
    #[inline(always)]
    pub const fn en(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Set to 1, the chip automatically exits from LPCD mode according to the set AutoWupCfg value, At the same time, the interrupt flag is given."]
    #[inline(always)]
    pub fn set_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
}
impl Default for LpcdAutoWupCfg {
    #[inline(always)]
    fn default() -> LpcdAutoWupCfg {
        LpcdAutoWupCfg(0)
    }
}
impl From<u8> for LpcdAutoWupCfg {
    fn from(val: u8) -> LpcdAutoWupCfg {
        LpcdAutoWupCfg(val)
    }
}
impl From<LpcdAutoWupCfg> for u8 {
    fn from(val: LpcdAutoWupCfg) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Tmode(pub u8);
impl Tmode {
    #[doc = ""]
    #[inline(always)]
    pub const fn tprescaler_hi(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_tprescaler_hi(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    #[doc = "Restarts the timer automatically after finished counting down to 0."]
    #[inline(always)]
    pub const fn tautorestart(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Restarts the timer automatically after finished counting down to 0."]
    #[inline(always)]
    pub fn set_tautorestart(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn tgated(&self) -> u8 {
        let val = (self.0 >> 5usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_tgated(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 5usize)) | (((val as u8) & 0x03) << 5usize);
    }
    #[doc = "Sets the Timer start/stop conditions to Auto mode."]
    #[inline(always)]
    pub const fn tauto(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Sets the Timer start/stop conditions to Auto mode."]
    #[inline(always)]
    pub fn set_tauto(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Tmode {
    #[inline(always)]
    fn default() -> Tmode {
        Tmode(0)
    }
}
impl From<u8> for Tmode {
    fn from(val: u8) -> Tmode {
        Tmode(val)
    }
}
impl From<Tmode> for u8 {
    fn from(val: Tmode) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Txsel(pub u8);
impl Txsel {
    #[doc = ""]
    #[inline(always)]
    pub const fn sigoutsel(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_sigoutsel(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn driversel(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_driversel(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 4usize)) | (((val as u8) & 0x03) << 4usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn loadmodsel(&self) -> u8 {
        let val = (self.0 >> 6usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_loadmodsel(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 6usize)) | (((val as u8) & 0x03) << 6usize);
    }
}
impl Default for Txsel {
    #[inline(always)]
    fn default() -> Txsel {
        Txsel(0)
    }
}
impl From<u8> for Txsel {
    fn from(val: u8) -> Txsel {
        Txsel(val)
    }
}
impl From<Txsel> for u8 {
    fn from(val: Txsel) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Txbitphase(pub u8);
impl Txbitphase {
    #[doc = ""]
    #[inline(always)]
    pub const fn txbitphase(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x7f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_txbitphase(&mut self, val: u8) {
        self.0 = (self.0 & !(0x7f << 0usize)) | (((val as u8) & 0x7f) << 0usize);
    }
    #[doc = "If 1 the receive clock may change between Rf and oscilator."]
    #[inline(always)]
    pub const fn rcvclkchange(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "If 1 the receive clock may change between Rf and oscilator."]
    #[inline(always)]
    pub fn set_rcvclkchange(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Txbitphase {
    #[inline(always)]
    fn default() -> Txbitphase {
        Txbitphase(0)
    }
}
impl From<u8> for Txbitphase {
    fn from(val: u8) -> Txbitphase {
        Txbitphase(val)
    }
}
impl From<Txbitphase> for u8 {
    fn from(val: Txbitphase) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct IrqBak(pub u8);
impl IrqBak {
    #[doc = "Irq Inv backup"]
    #[inline(always)]
    pub const fn irq_inv_bak(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "Irq Inv backup"]
    #[inline(always)]
    pub fn set_irq_inv_bak(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "Irq pushpull backup"]
    #[inline(always)]
    pub const fn irq_pushpull_bak(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "Irq pushpull backup"]
    #[inline(always)]
    pub fn set_irq_pushpull_bak(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
}
impl Default for IrqBak {
    #[inline(always)]
    fn default() -> IrqBak {
        IrqBak(0)
    }
}
impl From<u8> for IrqBak {
    fn from(val: u8) -> IrqBak {
        IrqBak(val)
    }
}
impl From<IrqBak> for u8 {
    fn from(val: IrqBak) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Rxtreshold(pub u8);
impl Rxtreshold {
    #[doc = ""]
    #[inline(always)]
    pub const fn collevel(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x07;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_collevel(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val as u8) & 0x07) << 0usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn minlevel(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x0f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_minlevel(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 4usize)) | (((val as u8) & 0x0f) << 4usize);
    }
}
impl Default for Rxtreshold {
    #[inline(always)]
    fn default() -> Rxtreshold {
        Rxtreshold(0)
    }
}
impl From<u8> for Rxtreshold {
    fn from(val: u8) -> Rxtreshold {
        Rxtreshold(val)
    }
}
impl From<Rxtreshold> for u8 {
    fn from(val: Rxtreshold) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Commien(pub u8);
impl Commien {
    #[doc = "Timer Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn timeri(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "Timer Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_timeri(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "Error Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn erri(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "Error Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_erri(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "LoAlert Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn loalerti(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "LoAlert Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_loalerti(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "HiAlert Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn hialerti(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "HiAlert Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_hialerti(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "Idle Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn idlei(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Idle Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_idlei(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Receive Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn rxi(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Receive Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_rxi(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = "Transmit Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn txi(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "Transmit Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_txi(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    #[doc = "Inverts the output of IRQ Pin."]
    #[inline(always)]
    pub const fn irqinv(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Inverts the output of IRQ Pin."]
    #[inline(always)]
    pub fn set_irqinv(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Commien {
    #[inline(always)]
    fn default() -> Commien {
        Commien(0)
    }
}
impl From<u8> for Commien {
    fn from(val: u8) -> Commien {
        Commien(val)
    }
}
impl From<Commien> for u8 {
    fn from(val: Commien) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Modgsp(pub u8);
impl Modgsp {
    #[doc = ""]
    #[inline(always)]
    pub const fn modgsp(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x3f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_modgsp(&mut self, val: u8) {
        self.0 = (self.0 & !(0x3f << 0usize)) | (((val as u8) & 0x3f) << 0usize);
    }
}
impl Default for Modgsp {
    #[inline(always)]
    fn default() -> Modgsp {
        Modgsp(0)
    }
}
impl From<u8> for Modgsp {
    fn from(val: u8) -> Modgsp {
        Modgsp(val)
    }
}
impl From<Modgsp> for u8 {
    fn from(val: Modgsp) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Rxsel(pub u8);
impl Rxsel {
    #[doc = ""]
    #[inline(always)]
    pub const fn rxwait(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x3f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_rxwait(&mut self, val: u8) {
        self.0 = (self.0 & !(0x3f << 0usize)) | (((val as u8) & 0x3f) << 0usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn uartsel(&self) -> u8 {
        let val = (self.0 >> 6usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_uartsel(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 6usize)) | (((val as u8) & 0x03) << 6usize);
    }
}
impl Default for Rxsel {
    #[inline(always)]
    fn default() -> Rxsel {
        Rxsel(0)
    }
}
impl From<u8> for Rxsel {
    fn from(val: u8) -> Rxsel {
        Rxsel(val)
    }
}
impl From<Rxsel> for u8 {
    fn from(val: Rxsel) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Commirq(pub u8);
impl Commirq {
    #[doc = "Timer Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn timeri(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "Timer Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_timeri(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "Error Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn erri(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "Error Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_erri(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "LoAlert Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn loalerti(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "LoAlert Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_loalerti(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "HiAlert Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn hialerti(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "HiAlert Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_hialerti(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "Idle Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn idlei(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Idle Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_idlei(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Receive Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn rxi(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Receive Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_rxi(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = "Transmit Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn txi(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "Transmit Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_txi(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    #[doc = "Bit position to set/clear dedicated IRQ bits."]
    #[inline(always)]
    pub const fn set(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Bit position to set/clear dedicated IRQ bits."]
    #[inline(always)]
    pub fn set_set(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Commirq {
    #[inline(always)]
    fn default() -> Commirq {
        Commirq(0)
    }
}
impl From<u8> for Commirq {
    fn from(val: u8) -> Commirq {
        Commirq(val)
    }
}
impl From<Commirq> for u8 {
    fn from(val: Commirq) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Command(pub u8);
impl Command {
    #[doc = "Run a command"]
    #[inline(always)]
    pub const fn command(&self) -> CommandVal {
        let val = (self.0 >> 0usize) & 0x0f;
        CommandVal(val as u8)
    }
    #[doc = "Run a command"]
    #[inline(always)]
    pub fn set_command(&mut self, val: CommandVal) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val.0 as u8) & 0x0f) << 0usize);
    }
    #[doc = "Switches FM175XX to Power Down mode."]
    #[inline(always)]
    pub const fn powerdown(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Switches FM175XX to Power Down mode."]
    #[inline(always)]
    pub fn set_powerdown(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Switches the receiver on/off."]
    #[inline(always)]
    pub const fn rcvoff(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Switches the receiver on/off."]
    #[inline(always)]
    pub fn set_rcvoff(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
}
impl Default for Command {
    #[inline(always)]
    fn default() -> Command {
        Command(0)
    }
}
impl From<u8> for Command {
    fn from(val: u8) -> Command {
        Command(val)
    }
}
impl From<Command> for u8 {
    fn from(val: Command) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Bitframing(pub u8);
impl Bitframing {
    #[doc = "Used for bitwise frame format transmission. Defines the number of bits that need to be sent in the last byte. 0 means that all bits of the last byte are to be sent."]
    #[inline(always)]
    pub const fn txlastbits(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x07;
        val as u8
    }
    #[doc = "Used for bitwise frame format transmission. Defines the number of bits that need to be sent in the last byte. 0 means that all bits of the last byte are to be sent."]
    #[inline(always)]
    pub fn set_txlastbits(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val as u8) & 0x07) << 0usize);
    }
    #[doc = "Used for receiving in bit frame format. RxAlign defines the data that needs to be stored in the FIFO to receive the first bit of data location. Subsequent received data bits are stored in the next position. This bit is only used for bitwise anticollision at 106 kbit/s. Set to 0 in other modes."]
    #[inline(always)]
    pub const fn rxalign(&self) -> u8 {
        let val = (self.0 >> 4usize) & 0x07;
        val as u8
    }
    #[doc = "Used for receiving in bit frame format. RxAlign defines the data that needs to be stored in the FIFO to receive the first bit of data location. Subsequent received data bits are stored in the next position. This bit is only used for bitwise anticollision at 106 kbit/s. Set to 0 in other modes."]
    #[inline(always)]
    pub fn set_rxalign(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 4usize)) | (((val as u8) & 0x07) << 4usize);
    }
    #[doc = "Starts transmission in transceive command if set to 1."]
    #[inline(always)]
    pub const fn startsend(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Starts transmission in transceive command if set to 1."]
    #[inline(always)]
    pub fn set_startsend(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Bitframing {
    #[inline(always)]
    fn default() -> Bitframing {
        Bitframing(0)
    }
}
impl From<u8> for Bitframing {
    fn from(val: u8) -> Bitframing {
        Bitframing(val)
    }
}
impl From<Bitframing> for u8 {
    fn from(val: Bitframing) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Divien(pub u8);
impl Divien {
    #[doc = "RF Off Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn rfoffi(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "RF Off Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_rfoffi(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "RF On Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn rfoni(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "RF On Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_rfoni(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "CRC Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn crci(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "CRC Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_crci(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "Mode Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn modei(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Mode Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_modei(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "SiginAct Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn siginact(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "SiginAct Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_siginact(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Sets the IRQ pin to Push Pull mode."]
    #[inline(always)]
    pub const fn irqpushpull(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Sets the IRQ pin to Push Pull mode."]
    #[inline(always)]
    pub fn set_irqpushpull(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Divien {
    #[inline(always)]
    fn default() -> Divien {
        Divien(0)
    }
}
impl From<u8> for Divien {
    fn from(val: u8) -> Divien {
        Divien(val)
    }
}
impl From<Divien> for u8 {
    fn from(val: Divien) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Rfcfg(pub u8);
impl Rfcfg {
    #[doc = "Sensitivity of the RF level detector"]
    #[inline(always)]
    pub const fn rflevel(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x0f;
        val as u8
    }
    #[doc = "Sensitivity of the RF level detector"]
    #[inline(always)]
    pub fn set_rflevel(&mut self, val: u8) {
        self.0 = (self.0 & !(0x0f << 0usize)) | (((val as u8) & 0x0f) << 0usize);
    }
    #[doc = "Receiver gain"]
    #[inline(always)]
    pub const fn rxgain(&self) -> Rxgain {
        let val = (self.0 >> 4usize) & 0x07;
        Rxgain(val as u8)
    }
    #[doc = "Receiver gain"]
    #[inline(always)]
    pub fn set_rxgain(&mut self, val: Rxgain) {
        self.0 = (self.0 & !(0x07 << 4usize)) | (((val.0 as u8) & 0x07) << 4usize);
    }
    #[doc = "Activates the RF Level detector amplifier."]
    #[inline(always)]
    pub const fn rflevelamp(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Activates the RF Level detector amplifier."]
    #[inline(always)]
    pub fn set_rflevelamp(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Rfcfg {
    #[inline(always)]
    fn default() -> Rfcfg {
        Rfcfg(0)
    }
}
impl From<u8> for Rfcfg {
    fn from(val: u8) -> Rfcfg {
        Rfcfg(val)
    }
}
impl From<Rfcfg> for u8 {
    fn from(val: Rfcfg) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdMisc(pub u8);
impl LpcdMisc {
    #[doc = "lPCD test mode"]
    #[inline(always)]
    pub const fn calib_vmid_en(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "lPCD test mode"]
    #[inline(always)]
    pub fn set_calib_vmid_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "LPCD amp en select"]
    #[inline(always)]
    pub const fn amp_en_sel(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "LPCD amp en select"]
    #[inline(always)]
    pub fn set_amp_en_sel(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
}
impl Default for LpcdMisc {
    #[inline(always)]
    fn default() -> LpcdMisc {
        LpcdMisc(0)
    }
}
impl From<u8> for LpcdMisc {
    fn from(val: u8) -> LpcdMisc {
        LpcdMisc(val)
    }
}
impl From<LpcdMisc> for u8 {
    fn from(val: LpcdMisc) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdRft2(pub u8);
impl LpcdRft2 {
    #[doc = "vdem of lpcd"]
    #[inline(always)]
    pub const fn aux2_vdem_lpcd(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "vdem of lpcd"]
    #[inline(always)]
    pub fn set_aux2_vdem_lpcd(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "voltage of charecap"]
    #[inline(always)]
    pub const fn aux2_vp_lpcd(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "voltage of charecap"]
    #[inline(always)]
    pub fn set_aux2_vp_lpcd(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
}
impl Default for LpcdRft2 {
    #[inline(always)]
    fn default() -> LpcdRft2 {
        LpcdRft2(0)
    }
}
impl From<u8> for LpcdRft2 {
    fn from(val: u8) -> LpcdRft2 {
        LpcdRft2(val)
    }
}
impl From<LpcdRft2> for u8 {
    fn from(val: LpcdRft2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Status1(pub u8);
impl Status1 {
    #[doc = "status LoAlert."]
    #[inline(always)]
    pub const fn loalert(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "status LoAlert."]
    #[inline(always)]
    pub fn set_loalert(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "status HiAlert."]
    #[inline(always)]
    pub const fn hialert(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "status HiAlert."]
    #[inline(always)]
    pub fn set_hialert(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "status RF is on/off."]
    #[inline(always)]
    pub const fn rfon(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "status RF is on/off."]
    #[inline(always)]
    pub fn set_rfon(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "status Timer is running."]
    #[inline(always)]
    pub const fn trunnung(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "status Timer is running."]
    #[inline(always)]
    pub fn set_trunnung(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "status IRQ is active."]
    #[inline(always)]
    pub const fn irq(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "status IRQ is active."]
    #[inline(always)]
    pub fn set_irq(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "status CRC Ready."]
    #[inline(always)]
    pub const fn crcready(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "status CRC Ready."]
    #[inline(always)]
    pub fn set_crcready(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = "status CRC OK."]
    #[inline(always)]
    pub const fn crcok(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "status CRC OK."]
    #[inline(always)]
    pub fn set_crcok(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
}
impl Default for Status1 {
    #[inline(always)]
    fn default() -> Status1 {
        Status1(0)
    }
}
impl From<u8> for Status1 {
    fn from(val: u8) -> Status1 {
        Status1(val)
    }
}
impl From<Status1> for u8 {
    fn from(val: Status1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdRft5(pub u8);
impl LpcdRft5 {
    #[doc = "lPCD test mode"]
    #[inline(always)]
    pub const fn testen(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "lPCD test mode"]
    #[inline(always)]
    pub fn set_testen(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "Auto wakeup test mode"]
    #[inline(always)]
    pub const fn awup_tsel(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "Auto wakeup test mode"]
    #[inline(always)]
    pub fn set_awup_tsel(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "RNG mode sel"]
    #[inline(always)]
    pub const fn rng_mode_sel(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "RNG mode sel"]
    #[inline(always)]
    pub fn set_rng_mode_sel(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "use retention mode"]
    #[inline(always)]
    pub const fn use_ret(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "use retention mode"]
    #[inline(always)]
    pub fn set_use_ret(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
}
impl Default for LpcdRft5 {
    #[inline(always)]
    fn default() -> LpcdRft5 {
        LpcdRft5(0)
    }
}
impl From<u8> for LpcdRft5 {
    fn from(val: u8) -> LpcdRft5 {
        LpcdRft5(val)
    }
}
impl From<LpcdRft5> for u8 {
    fn from(val: LpcdRft5) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Autotest(pub u8);
impl Autotest {
    #[doc = ""]
    #[inline(always)]
    pub const fn amprcv(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_amprcv(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
}
impl Default for Autotest {
    #[inline(always)]
    fn default() -> Autotest {
        Autotest(0)
    }
}
impl From<u8> for Autotest {
    fn from(val: u8) -> Autotest {
        Autotest(val)
    }
}
impl From<Autotest> for u8 {
    fn from(val: Autotest) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Txmode(pub u8);
impl Txmode {
    #[inline(always)]
    pub const fn framing(&self) -> Framing {
        let val = (self.0 >> 0usize) & 0x03;
        Framing(val as u8)
    }
    #[inline(always)]
    pub fn set_framing(&mut self, val: Framing) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val.0 as u8) & 0x03) << 0usize);
    }
    #[doc = "Activates TXMix functionality."]
    #[inline(always)]
    pub const fn txmix(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "Activates TXMix functionality."]
    #[inline(always)]
    pub fn set_txmix(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "Activates inverted transmission mode."]
    #[inline(always)]
    pub const fn invmod(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Activates inverted transmission mode."]
    #[inline(always)]
    pub fn set_invmod(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[inline(always)]
    pub const fn speed(&self) -> Speed {
        let val = (self.0 >> 4usize) & 0x07;
        Speed(val as u8)
    }
    #[inline(always)]
    pub fn set_speed(&mut self, val: Speed) {
        self.0 = (self.0 & !(0x07 << 4usize)) | (((val.0 as u8) & 0x07) << 4usize);
    }
    #[doc = "Activates transmit or receive CRC."]
    #[inline(always)]
    pub const fn crcen(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Activates transmit or receive CRC."]
    #[inline(always)]
    pub fn set_crcen(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Txmode {
    #[inline(always)]
    fn default() -> Txmode {
        Txmode(0)
    }
}
impl From<u8> for Txmode {
    fn from(val: u8) -> Txmode {
        Txmode(val)
    }
}
impl From<Txmode> for u8 {
    fn from(val: Txmode) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Manualrcv(pub u8);
impl Manualrcv {
    #[doc = ""]
    #[inline(always)]
    pub const fn hpcf(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_hpcf(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val as u8) & 0x03) << 0usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn manualhpcf(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_manualhpcf(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn largebwpll(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_largebwpll(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "Disables the parity generation and sending independent from the mode."]
    #[inline(always)]
    pub const fn paritydisable(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Disables the parity generation and sending independent from the mode."]
    #[inline(always)]
    pub fn set_paritydisable(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
}
impl Default for Manualrcv {
    #[inline(always)]
    fn default() -> Manualrcv {
        Manualrcv(0)
    }
}
impl From<u8> for Manualrcv {
    fn from(val: u8) -> Manualrcv {
        Manualrcv(val)
    }
}
impl From<Manualrcv> for u8 {
    fn from(val: Manualrcv) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Coll(pub u8);
impl Coll {
    #[doc = "Set to 1 if no collision is detected or the collision position is outside the range of CollPos. This bit is only Interpreted in ISO/IEC 14443A reader mode."]
    #[inline(always)]
    pub const fn collpos(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x1f;
        val as u8
    }
    #[doc = "Set to 1 if no collision is detected or the collision position is outside the range of CollPos. This bit is only Interpreted in ISO/IEC 14443A reader mode."]
    #[inline(always)]
    pub fn set_collpos(&mut self, val: u8) {
        self.0 = (self.0 & !(0x1f << 0usize)) | (((val as u8) & 0x1f) << 0usize);
    }
    #[doc = "Set to 1 if no collision is detected or the collision position is outside the range of CollPos. This bit is only Interpreted in ISO/IEC 14443A reader mode."]
    #[inline(always)]
    pub const fn collposnotvalid(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Set to 1 if no collision is detected or the collision position is outside the range of CollPos. This bit is only Interpreted in ISO/IEC 14443A reader mode."]
    #[inline(always)]
    pub fn set_collposnotvalid(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = "Activates mode to keep data after collision."]
    #[inline(always)]
    pub const fn valuesaftercoll(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Activates mode to keep data after collision."]
    #[inline(always)]
    pub fn set_valuesaftercoll(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Coll {
    #[inline(always)]
    fn default() -> Coll {
        Coll(0)
    }
}
impl From<u8> for Coll {
    fn from(val: u8) -> Coll {
        Coll(val)
    }
}
impl From<Coll> for u8 {
    fn from(val: Coll) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Divirq(pub u8);
impl Divirq {
    #[doc = "RF Off Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn rfoffi(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "RF Off Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_rfoffi(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "RF On Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn rfoni(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "RF On Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_rfoni(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "CRC Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn crci(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "CRC Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_crci(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "Mode Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn modei(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Mode Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_modei(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "SiginAct Interrupt Enable/Request."]
    #[inline(always)]
    pub const fn siginact(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "SiginAct Interrupt Enable/Request."]
    #[inline(always)]
    pub fn set_siginact(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Bit position to set/clear dedicated IRQ bits."]
    #[inline(always)]
    pub const fn set(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Bit position to set/clear dedicated IRQ bits."]
    #[inline(always)]
    pub fn set_set(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Divirq {
    #[inline(always)]
    fn default() -> Divirq {
        Divirq(0)
    }
}
impl From<u8> for Divirq {
    fn from(val: u8) -> Divirq {
        Divirq(val)
    }
}
impl From<Divirq> for u8 {
    fn from(val: Divirq) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdRft3(pub u8);
impl LpcdRft3 {
    #[doc = "enable lp osc10k"]
    #[inline(always)]
    pub const fn lp_osc10k_en(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "enable lp osc10k"]
    #[inline(always)]
    pub fn set_lp_osc10k_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "enable lp osc10k calibra mode"]
    #[inline(always)]
    pub const fn lp_osc_calibra_en(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "enable lp osc10k calibra mode"]
    #[inline(always)]
    pub fn set_lp_osc_calibra_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "enable lp t1 current test"]
    #[inline(always)]
    pub const fn lp_curr_test(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "enable lp t1 current test"]
    #[inline(always)]
    pub fn set_lp_curr_test(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "lpcd_test2\\[3\\]:LPCD_OUT"]
    #[inline(always)]
    pub const fn test2_lpcd_out(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "lpcd_test2\\[3\\]:LPCD_OUT"]
    #[inline(always)]
    pub fn set_test2_lpcd_out(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
}
impl Default for LpcdRft3 {
    #[inline(always)]
    fn default() -> LpcdRft3 {
        LpcdRft3(0)
    }
}
impl From<u8> for LpcdRft3 {
    fn from(val: u8) -> LpcdRft3 {
        LpcdRft3(val)
    }
}
impl From<LpcdRft3> for u8 {
    fn from(val: LpcdRft3) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdCtrl2(pub u8);
impl LpcdCtrl2 {
    #[doc = "Configure the drive control of the RF field P tube in LPCD mode. From 000 to 111, the driving capacity increases from small to large."]
    #[inline(always)]
    pub const fn cwp(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x07;
        val as u8
    }
    #[doc = "Configure the drive control of the RF field P tube in LPCD mode. From 000 to 111, the driving capacity increases from small to large."]
    #[inline(always)]
    pub fn set_cwp(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 0usize)) | (((val as u8) & 0x07) << 0usize);
    }
    #[doc = "Configure the drive control of the RF field N tube in LPCD mode. 0- Small drive, 1- Big drive"]
    #[inline(always)]
    pub const fn cwn(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Configure the drive control of the RF field N tube in LPCD mode. 0- Small drive, 1- Big drive"]
    #[inline(always)]
    pub fn set_cwn(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "1 which means that the TX2 output is enabled, and the output is inverse to TX1."]
    #[inline(always)]
    pub const fn tx2en(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "1 which means that the TX2 output is enabled, and the output is inverse to TX1."]
    #[inline(always)]
    pub fn set_tx2en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
}
impl Default for LpcdCtrl2 {
    #[inline(always)]
    fn default() -> LpcdCtrl2 {
        LpcdCtrl2(0)
    }
}
impl From<u8> for LpcdCtrl2 {
    fn from(val: u8) -> LpcdCtrl2 {
        LpcdCtrl2(val)
    }
}
impl From<LpcdCtrl2> for u8 {
    fn from(val: LpcdCtrl2) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdCtrl1(pub u8);
impl LpcdCtrl1 {
    #[doc = "enble LPCD"]
    #[inline(always)]
    pub const fn en(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "enble LPCD"]
    #[inline(always)]
    pub fn set_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "lpcd reset"]
    #[inline(always)]
    pub const fn rstn(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "lpcd reset"]
    #[inline(always)]
    pub fn set_rstn(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "into lpcd calibra mode"]
    #[inline(always)]
    pub const fn calibra_en(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "into lpcd calibra mode"]
    #[inline(always)]
    pub fn set_calibra_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "Compare times 1 or 3"]
    #[inline(always)]
    pub const fn sense_1(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Compare times 1 or 3"]
    #[inline(always)]
    pub fn set_sense_1(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "Enable LPCD IE"]
    #[inline(always)]
    pub const fn ie(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Enable LPCD IE"]
    #[inline(always)]
    pub fn set_ie(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Lpcd register Bit ctrl set bit"]
    #[inline(always)]
    pub const fn bit_ctrl_set(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Lpcd register Bit ctrl set bit"]
    #[inline(always)]
    pub fn set_bit_ctrl_set(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
}
impl Default for LpcdCtrl1 {
    #[inline(always)]
    fn default() -> LpcdCtrl1 {
        LpcdCtrl1(0)
    }
}
impl From<u8> for LpcdCtrl1 {
    fn from(val: u8) -> LpcdCtrl1 {
        LpcdCtrl1(val)
    }
}
impl From<LpcdCtrl1> for u8 {
    fn from(val: LpcdCtrl1) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Mifare(pub u8);
impl Mifare {
    #[doc = ""]
    #[inline(always)]
    pub const fn txwait(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_txwait(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 0usize)) | (((val as u8) & 0x03) << 0usize);
    }
    #[doc = "Configures the internal state machine only to answer to Wakeup commands according to ISO 14443-3."]
    #[inline(always)]
    pub const fn mfhalted(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "Configures the internal state machine only to answer to Wakeup commands according to ISO 14443-3."]
    #[inline(always)]
    pub fn set_mfhalted(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn taumiller(&self) -> u8 {
        let val = (self.0 >> 3usize) & 0x03;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_taumiller(&mut self, val: u8) {
        self.0 = (self.0 & !(0x03 << 3usize)) | (((val as u8) & 0x03) << 3usize);
    }
    #[doc = ""]
    #[inline(always)]
    pub const fn sensmiller(&self) -> u8 {
        let val = (self.0 >> 5usize) & 0x07;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_sensmiller(&mut self, val: u8) {
        self.0 = (self.0 & !(0x07 << 5usize)) | (((val as u8) & 0x07) << 5usize);
    }
}
impl Default for Mifare {
    #[inline(always)]
    fn default() -> Mifare {
        Mifare(0)
    }
}
impl From<u8> for Mifare {
    fn from(val: u8) -> Mifare {
        Mifare(val)
    }
}
impl From<Mifare> for u8 {
    fn from(val: Mifare) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Txauto(pub u8);
impl Txauto {
    #[doc = "Switches on the driver one automatically according to the other settings."]
    #[inline(always)]
    pub const fn tx1rfautoen(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "Switches on the driver one automatically according to the other settings."]
    #[inline(always)]
    pub fn set_tx1rfautoen(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "Switches on the driver two automatically according to the other settings."]
    #[inline(always)]
    pub const fn tx2rfautoen(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "Switches on the driver two automatically according to the other settings."]
    #[inline(always)]
    pub fn set_tx2rfautoen(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "Activate the initial RF on procedure as defined iun ECMA-340."]
    #[inline(always)]
    pub const fn initialrfon(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "Activate the initial RF on procedure as defined iun ECMA-340."]
    #[inline(always)]
    pub fn set_initialrfon(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "Activates the automatic time jitter generation by switching on the Rf field as defined in ECMA-340."]
    #[inline(always)]
    pub const fn caon(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Activates the automatic time jitter generation by switching on the Rf field as defined in ECMA-340."]
    #[inline(always)]
    pub fn set_caon(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "Activates automatic wakeup of the FM175XX if set to 1."]
    #[inline(always)]
    pub const fn autowakeup(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Activates automatic wakeup of the FM175XX if set to 1."]
    #[inline(always)]
    pub fn set_autowakeup(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = "Activates 100%ASK mode independent of driver settings."]
    #[inline(always)]
    pub const fn force100ask(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "Activates 100%ASK mode independent of driver settings."]
    #[inline(always)]
    pub fn set_force100ask(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    #[doc = "Switches the RF automatically off after transmission is finished."]
    #[inline(always)]
    pub const fn autorfoff(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Switches the RF automatically off after transmission is finished."]
    #[inline(always)]
    pub fn set_autorfoff(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Txauto {
    #[inline(always)]
    fn default() -> Txauto {
        Txauto(0)
    }
}
impl From<u8> for Txauto {
    fn from(val: u8) -> Txauto {
        Txauto(val)
    }
}
impl From<Txauto> for u8 {
    fn from(val: Txauto) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Fifolevel(pub u8);
impl Fifolevel {
    #[doc = "FIFO level, in bytes. 0..=64"]
    #[inline(always)]
    pub const fn level(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x7f;
        val as u8
    }
    #[doc = "FIFO level, in bytes. 0..=64"]
    #[inline(always)]
    pub fn set_level(&mut self, val: u8) {
        self.0 = (self.0 & !(0x7f << 0usize)) | (((val as u8) & 0x7f) << 0usize);
    }
    #[doc = "Clears FIFO buffer if set to 1"]
    #[inline(always)]
    pub const fn flushfifo(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Clears FIFO buffer if set to 1"]
    #[inline(always)]
    pub fn set_flushfifo(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Fifolevel {
    #[inline(always)]
    fn default() -> Fifolevel {
        Fifolevel(0)
    }
}
impl From<u8> for Fifolevel {
    fn from(val: u8) -> Fifolevel {
        Fifolevel(val)
    }
}
impl From<Fifolevel> for u8 {
    fn from(val: Fifolevel) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct LpcdRft4(pub u8);
impl LpcdRft4 {
    #[doc = "D5:T1_OUT"]
    #[inline(always)]
    pub const fn t1_out_en(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "D5:T1_OUT"]
    #[inline(always)]
    pub fn set_t1_out_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "D4:OSC_CLK_OUT"]
    #[inline(always)]
    pub const fn oscclk_out_en(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "D4:OSC_CLK_OUT"]
    #[inline(always)]
    pub fn set_oscclk_out_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "D3:OSC_EN"]
    #[inline(always)]
    pub const fn oscen_out_en(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "D3:OSC_EN"]
    #[inline(always)]
    pub fn set_oscen_out_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "D2:LP_CLK or LPCD_OUT"]
    #[inline(always)]
    pub const fn lp_clk_lpcd_out_en(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "D2:LP_CLK or LPCD_OUT"]
    #[inline(always)]
    pub fn set_lp_clk_lpcd_out_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "D1:T3_OUT"]
    #[inline(always)]
    pub const fn t3_out_en(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "D1:T3_OUT"]
    #[inline(always)]
    pub fn set_t3_out_en(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
}
impl Default for LpcdRft4 {
    #[inline(always)]
    fn default() -> LpcdRft4 {
        LpcdRft4(0)
    }
}
impl From<u8> for LpcdRft4 {
    fn from(val: u8) -> LpcdRft4 {
        LpcdRft4(val)
    }
}
impl From<LpcdRft4> for u8 {
    fn from(val: LpcdRft4) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Cwgsp(pub u8);
impl Cwgsp {
    #[doc = ""]
    #[inline(always)]
    pub const fn cwgsp(&self) -> u8 {
        let val = (self.0 >> 0usize) & 0x3f;
        val as u8
    }
    #[doc = ""]
    #[inline(always)]
    pub fn set_cwgsp(&mut self, val: u8) {
        self.0 = (self.0 & !(0x3f << 0usize)) | (((val as u8) & 0x3f) << 0usize);
    }
}
impl Default for Cwgsp {
    #[inline(always)]
    fn default() -> Cwgsp {
        Cwgsp(0)
    }
}
impl From<u8> for Cwgsp {
    fn from(val: u8) -> Cwgsp {
        Cwgsp(val)
    }
}
impl From<Cwgsp> for u8 {
    fn from(val: Cwgsp) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Txcontrol(pub u8);
impl Txcontrol {
    #[doc = "Switches the driver for Tx1 pin on."]
    #[inline(always)]
    pub const fn tx1rfen(&self) -> bool {
        let val = (self.0 >> 0usize) & 0x01;
        val != 0
    }
    #[doc = "Switches the driver for Tx1 pin on."]
    #[inline(always)]
    pub fn set_tx1rfen(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 0usize)) | (((val as u8) & 0x01) << 0usize);
    }
    #[doc = "Switches the driver for Tx2 pin on."]
    #[inline(always)]
    pub const fn tx2rfen(&self) -> bool {
        let val = (self.0 >> 1usize) & 0x01;
        val != 0
    }
    #[doc = "Switches the driver for Tx2 pin on."]
    #[inline(always)]
    pub fn set_tx2rfen(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 1usize)) | (((val as u8) & 0x01) << 1usize);
    }
    #[doc = "Does not activate the driver if an external RF is detected. Only valid in combination with JBIT_TX2RFEN and JBIT_TX1RFEN."]
    #[inline(always)]
    pub const fn checkrf(&self) -> bool {
        let val = (self.0 >> 2usize) & 0x01;
        val != 0
    }
    #[doc = "Does not activate the driver if an external RF is detected. Only valid in combination with JBIT_TX2RFEN and JBIT_TX1RFEN."]
    #[inline(always)]
    pub fn set_checkrf(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 2usize)) | (((val as u8) & 0x01) << 2usize);
    }
    #[doc = "Does not modulate the Tx2 output, only constant wave."]
    #[inline(always)]
    pub const fn tx2cw(&self) -> bool {
        let val = (self.0 >> 3usize) & 0x01;
        val != 0
    }
    #[doc = "Does not modulate the Tx2 output, only constant wave."]
    #[inline(always)]
    pub fn set_tx2cw(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 3usize)) | (((val as u8) & 0x01) << 3usize);
    }
    #[doc = "Inverts the Tx1 output if drivers are switched off."]
    #[inline(always)]
    pub const fn invtx1off(&self) -> bool {
        let val = (self.0 >> 4usize) & 0x01;
        val != 0
    }
    #[doc = "Inverts the Tx1 output if drivers are switched off."]
    #[inline(always)]
    pub fn set_invtx1off(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 4usize)) | (((val as u8) & 0x01) << 4usize);
    }
    #[doc = "Inverts the Tx2 output if drivers are switched off."]
    #[inline(always)]
    pub const fn invtx2off(&self) -> bool {
        let val = (self.0 >> 5usize) & 0x01;
        val != 0
    }
    #[doc = "Inverts the Tx2 output if drivers are switched off."]
    #[inline(always)]
    pub fn set_invtx2off(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 5usize)) | (((val as u8) & 0x01) << 5usize);
    }
    #[doc = "Inverts the Tx1 output if drivers are switched on."]
    #[inline(always)]
    pub const fn invtx1on(&self) -> bool {
        let val = (self.0 >> 6usize) & 0x01;
        val != 0
    }
    #[doc = "Inverts the Tx1 output if drivers are switched on."]
    #[inline(always)]
    pub fn set_invtx1on(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 6usize)) | (((val as u8) & 0x01) << 6usize);
    }
    #[doc = "Inverts the Tx2 output if drivers are switched on."]
    #[inline(always)]
    pub const fn invtx2on(&self) -> bool {
        let val = (self.0 >> 7usize) & 0x01;
        val != 0
    }
    #[doc = "Inverts the Tx2 output if drivers are switched on."]
    #[inline(always)]
    pub fn set_invtx2on(&mut self, val: bool) {
        self.0 = (self.0 & !(0x01 << 7usize)) | (((val as u8) & 0x01) << 7usize);
    }
}
impl Default for Txcontrol {
    #[inline(always)]
    fn default() -> Txcontrol {
        Txcontrol(0)
    }
}
impl From<u8> for Txcontrol {
    fn from(val: u8) -> Txcontrol {
        Txcontrol(val)
    }
}
impl From<Txcontrol> for u8 {
    fn from(val: Txcontrol) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Speed(pub u8);
impl Speed {
    #[doc = "106kbps"]
    pub const _106KBPS: Self = Self(0);
    #[doc = "212kbps"]
    pub const _212KBPS: Self = Self(0x01);
    #[doc = "424kbps"]
    pub const _424KBPS: Self = Self(0x02);
    #[doc = "848kbps"]
    pub const _848KBPS: Self = Self(0x03);
    #[doc = "1.6Mbps"]
    pub const _1_6MBPS: Self = Self(0x04);
    #[doc = "3.3Mbps"]
    pub const _3_2MBPS: Self = Self(0x05);
}
impl From<u8> for Speed {
    fn from(val: u8) -> Speed {
        Speed(val)
    }
}
impl From<Speed> for u8 {
    fn from(val: Speed) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Framing(pub u8);
impl Framing {
    #[doc = "ISO14443A communication mode."]
    pub const ISO14443A: Self = Self(0);
    #[doc = "NFC/Active communication mode."]
    pub const NFC: Self = Self(0x01);
    #[doc = "FeliCa communication mode."]
    pub const FELICA: Self = Self(0x02);
    #[doc = "ISO14443B communication mode."]
    pub const ISO14443B: Self = Self(0x03);
}
impl From<u8> for Framing {
    fn from(val: u8) -> Framing {
        Framing(val)
    }
}
impl From<Framing> for u8 {
    fn from(val: Framing) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CommandVal(pub u8);
impl CommandVal {
    #[doc = "No action, cancel the current command execution"]
    pub const IDLE: Self = Self(0);
    #[doc = "Configure FM17550 as ISO14443A card analog communication mode"]
    pub const CONFIGURE: Self = Self(0x01);
    #[doc = "Generate a 10-byte random number"]
    pub const GENERATERANDOMID: Self = Self(0x02);
    #[doc = "Activate CRC coprocessor"]
    pub const CALCCRC: Self = Self(0x03);
    #[doc = "Transmit Data in transmit FIFO buffer"]
    pub const TRANSMIT: Self = Self(0x04);
    #[doc = "The instruction does not change and can be used to modify the instruction without affecting the instruction CommandReg registers, such as the PowerDown bit"]
    pub const NOCMDCHANGE: Self = Self(0x07);
    #[doc = "Activate the receiver circuit"]
    pub const RECEIVE: Self = Self(0x08);
    #[doc = "Transmit the data in the FIFO buffer to the antenna and automatically activate the receiver after transmission"]
    pub const TRANSCEIVE: Self = Self(0x0c);
    #[doc = "Handle ISO14443A anti-collision process (only supports card emulation mode)"]
    pub const AUTOCOLL: Self = Self(0x0d);
    #[doc = "Perform M1 security authentication as reader mode"]
    pub const AUTHENT: Self = Self(0x0e);
    #[doc = "Reset the FM17550"]
    pub const SOFTRESET: Self = Self(0x0f);
}
impl From<u8> for CommandVal {
    fn from(val: u8) -> CommandVal {
        CommandVal(val)
    }
}
impl From<CommandVal> for u8 {
    fn from(val: CommandVal) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LpcdGain(pub u8);
impl LpcdGain {
    #[doc = "2/5x"]
    pub const X2DIV5: Self = Self(0);
    #[doc = "2/3x"]
    pub const X2DIV3: Self = Self(0x01);
    #[doc = "1/2x"]
    pub const X1DIV2: Self = Self(0x02);
    #[doc = "1x"]
    pub const X1: Self = Self(0x03);
}
impl From<u8> for LpcdGain {
    fn from(val: u8) -> LpcdGain {
        LpcdGain(val)
    }
}
impl From<LpcdGain> for u8 {
    fn from(val: LpcdGain) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LpcdAutoWupTime(pub u8);
impl LpcdAutoWupTime {
    #[doc = "6 seconds"]
    pub const _6SEC: Self = Self(0);
    #[doc = "12 seconds"]
    pub const _12SEC: Self = Self(0x01);
    #[doc = "15 min"]
    pub const _15MIN: Self = Self(0x02);
    #[doc = "30 min"]
    pub const _30MIN: Self = Self(0x03);
    #[doc = "1 hour"]
    pub const _1HOUR: Self = Self(0x04);
    #[doc = "1.8 hours"]
    pub const _1_8HOUR: Self = Self(0x05);
    #[doc = "3.6 hours"]
    pub const _3_6HOUR: Self = Self(0x06);
    #[doc = "7.2 hours"]
    pub const _7_2HOUR: Self = Self(0x07);
}
impl From<u8> for LpcdAutoWupTime {
    fn from(val: u8) -> LpcdAutoWupTime {
        LpcdAutoWupTime(val)
    }
}
impl From<LpcdAutoWupTime> for u8 {
    fn from(val: LpcdAutoWupTime) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LpcdT3clkdivk(pub u8);
impl LpcdT3clkdivk {
    pub const DIV4: Self = Self(0);
    pub const DIV8: Self = Self(0x01);
    pub const DIV16: Self = Self(0x02);
}
impl From<u8> for LpcdT3clkdivk {
    fn from(val: u8) -> LpcdT3clkdivk {
        LpcdT3clkdivk(val)
    }
}
impl From<LpcdT3clkdivk> for u8 {
    fn from(val: LpcdT3clkdivk) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Rxgain(pub u8);
impl Rxgain {
    #[doc = "18 dB"]
    pub const _18DB: Self = Self(0);
    #[doc = "23 dB"]
    pub const _23DB: Self = Self(0x01);
    #[doc = "18 dB (dupe value)"]
    pub const _18DB_DUP: Self = Self(0x02);
    #[doc = "23 dB (dupe value)"]
    pub const _23DB_DUP: Self = Self(0x03);
    #[doc = "33 dB"]
    pub const _33DB: Self = Self(0x04);
    #[doc = "38 dB"]
    pub const _38DB: Self = Self(0x05);
    #[doc = "43 dB"]
    pub const _43DB: Self = Self(0x06);
    #[doc = "48 dB"]
    pub const _48DB: Self = Self(0x07);
}
impl From<u8> for Rxgain {
    fn from(val: u8) -> Rxgain {
        Rxgain(val)
    }
}
impl From<Rxgain> for u8 {
    fn from(val: Rxgain) -> u8 {
        val.0
    }
}
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct LpcdAttenuation(pub u8);
impl LpcdAttenuation {
    #[doc = "1.0x"]
    pub const X1_0: Self = Self(0);
    #[doc = "1.2x"]
    pub const X1_2: Self = Self(0x01);
    #[doc = "1.45x"]
    pub const X1_45: Self = Self(0x02);
    #[doc = "2.0x"]
    pub const X2_0: Self = Self(0x03);
    #[doc = "2.28x"]
    pub const X2_28: Self = Self(0x04);
    #[doc = "2.6x"]
    pub const X2_6: Self = Self(0x05);
    #[doc = "3.2x"]
    pub const X3_2: Self = Self(0x06);
    #[doc = "4.0x"]
    pub const X4_0: Self = Self(0x07);
}
impl From<u8> for LpcdAttenuation {
    fn from(val: u8) -> LpcdAttenuation {
        LpcdAttenuation(val)
    }
}
impl From<LpcdAttenuation> for u8 {
    fn from(val: LpcdAttenuation) -> u8 {
        val.0
    }
}
