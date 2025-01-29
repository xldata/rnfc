/*
Port of the ST HAL's C lib AAT, not RNFC's
*/

use embassy_time::{Duration, Timer};
use embedded_hal::digital::InputPin;
use embedded_hal_async::digital::Wait;

use crate::regs::*;
use crate::{Interface, St25r39};

const ST25R3916_AAT_CAP_DELAY_MAX: u64 = 10; /* Max Variable Capacitor settle delay */

/* Some constants that are not in RNFC lib but needed for ST C lib AAT port */
const ST25R3916_REG_OP_CONTROL_rx_chn: u8 = 1 << 5;
const ST25R3916_REG_MODE_om_iso14443a: u8 = 1 << 3;
const ST25R3916_REG_MODE_targ_init: u8 = 0 << 7;
const ST25R3916_REG_MODE_tr_am_ook: u8 = 0 << 2;
const ST25R3916_REG_MODE_nfc_ar_off: u8 = 0 << 0;
const ST25R3916_REG_RX_CONF1_ch_sel_AM: u8 = 0 << 7;
const ST25R3916_REG_RX_CONF2_demod_mode: u8 = 1 << 7;
const ST25R3916_REG_RX_CONF2_amd_sel: u8 = 1 << 6;
const ST25R3916_REG_RX_CONF2_amd_sel_peak: u8 = 0 << 6;
const ST25R3916_REG_AUX_MOD_rgs_am: u8 = 1 << 2; // For ST25R3916B. TODO this is in reg space B, check if properly written by RNFC

pub struct AatConfig {
    a_min: u8,
    a_max: u8,
    a_start: u8,
    a_step: u8,
    b_min: u8,
    b_max: u8,
    b_start: u8,
    b_step: u8,
    pha_target: u8,
    pha_weight: u16,
    amp_target: u8,
    amp_weight: u16,
    do_dynamic_steps: bool,
    measure_limit: u8,
}

impl AatConfig {
    fn new() -> Self {
        // default values from ST.com RFAL C lib
        Self {
            a_min: 0,
            a_max: 255,
            a_start: 127,
            a_step: 32,
            b_min: 0,
            b_max: 255,
            b_start: 127,
            b_step: 32,
            pha_target: 128,
            pha_weight: 2,
            amp_target: 196,
            amp_weight: 1,
            do_dynamic_steps: true,
            measure_limit: 50,
        }
    }
}

/*
 * struct representing out parameters for the antenna tuning
 */
pub struct St25r3916AatTuneResult {
    aat_a: u8,      /* serial cap after tuning */
    aat_b: u8,      /* parallel cap after tuning */
    pha: u8,        /* phase after tuning */
    amp: u8,        /* amplitude after tuning */
    measureCnt: u8, /* number of measures performed */
}

/*
 *****************************************************************************
 *  \brief  Perform antenna tuning
 *
 *  This function starts an antenna tuning procedure by modifying the serial
 *  and parallel capacitors of the antenna matching circuit via the AAT_A
 *  and AAT_B registers.
 *  This function is best run if the field is already turned on.
 *
 *  When used on ST25R3916B with new rgs_am=1 it is necessary to turn on the
 *  field before running this procedure or to set rgs_txonoff=0.
 *
 *****************************************************************************
 */

// TO-DO: discuss if we need the option to call func with other tuning params and a different initial result, like C lib has
// tune_antenna is the pub function to call from other places in the code, it's structured like the ST RFAL C lib including how it does its internal function calls

pub enum AntennaTuningError {
    NoChange,
    InvalidDirection,
    //MeasurementFailed,
}

pub trait AntennaTuning {
    async fn tune_antenna(&mut self) -> Result<(), AntennaTuningError>;
    async fn rfal_chip_measure_amplitude(&mut self, amp: &mut u8) -> ();
    async fn aat_measure(&mut self, a: &u8, b: &u8, amp: &mut u8, phs: &mut u8, ts: &mut St25r3916AatTuneResult) -> ();
    fn aat_calc_f(&mut self, tp: &AatConfig, amp: u8, pha: u8) -> u16;
    async fn aat_hill_climb(&mut self, tp: AatConfig, ts: St25r3916AatTuneResult);
    async fn aat_steepest_descent(
        &mut self,
        f_min: &mut u16,
        tp: &AatConfig,
        ts: &mut St25r3916AatTuneResult,
        direction: i32,
        neg_direction: i32,
    ) -> i32;
    async fn aat_greedy_descent(
        &mut self,
        f_min: &mut u16,
        tp: &AatConfig,
        ts: &mut St25r3916AatTuneResult,
        direction: i32,
    ) -> i32;
    fn aat_step_dac_vals(&mut self, tp: &AatConfig, a: &mut u8, b: &mut u8, dir: i32) -> Result<(), AntennaTuningError>;
}

impl<I, IrqPin> AntennaTuning for St25r39<I, IrqPin>
where
    I: Interface,
    IrqPin: InputPin + Wait,
{
    async fn tune_antenna(&mut self) -> Result<(), AntennaTuningError> {
        let tp = AatConfig::new();

        let ts = St25r3916AatTuneResult {
            aat_a: tp.a_start, /* serial cap after tuning */
            aat_b: tp.b_start, /* parallel cap after tuning */
            pha: 0,            /* phase after tuning */
            amp: 0,            /* amplitude after tuning */
            measureCnt: 0,     /* number of measures performed */
        };

        info!("Calling hill climb");
        self.aat_hill_climb(tp, ts).await;

        Ok(())
    }

    async fn aat_hill_climb(&mut self, mut tp: AatConfig, mut ts: St25r3916AatTuneResult) {
        let mut f_min: u16;
        let mut direction: i32;
        let mut gdirection: i32;
        let mut amp: u8 = 0;
        let mut phs: u8 = 0;

        /* Get a proper start value */
        self.aat_measure(&ts.aat_a.clone(), &ts.aat_b.clone(), &mut amp, &mut phs, &mut ts)
            .await;
        f_min = self.aat_calc_f(&tp, amp, phs);

        info!(
            "Start hill climb before outer loop ts.aat_a: {}, ts.aat_b: {}, f_min: {}",
            ts.aat_a, ts.aat_b, f_min
        );

        'outer: while tp.do_dynamic_steps && ((tp.a_step > 0) || (tp.b_step > 0)) {
            direction = 0; /* Initially and after reducing step sizes we don't have a previous direction */

            loop {
                info!("aat_measure loop");
                /* With the greedy step below always executed aftwards the -direction does never need to be investigated */
                direction = self
                    .aat_steepest_descent(&mut f_min, &tp, &mut ts, direction, -direction)
                    .await;
                info!("Direction from steepest_descent: {}", direction);

                if ts.measureCnt > tp.measure_limit {
                    info!("AAT steep hill climb stopped due to measure count > measure limit");
                    break 'outer;
                }

                loop {
                    gdirection = self.aat_greedy_descent(&mut f_min, &tp, &mut ts, direction).await;
                    info!("Direction from greedy descent: {}", gdirection);
                    if ts.measureCnt > tp.measure_limit {
                        info!("AAT greedy hill climb stopped due to measure count > measure limit");
                        break 'outer;
                    }

                    if gdirection == 0 {
                        break;
                    }
                }

                if direction == 0 {
                    break;
                }
            }

            info!("New steps, a: {}, b: {}", tp.a_step, tp.b_step);

            tp.a_step /= 2;
            tp.b_step /= 2;
        }
    }

    async fn aat_steepest_descent(
        &mut self,
        f_min: &mut u16,
        tp: &AatConfig,
        ts: &mut St25r3916AatTuneResult,
        direction: i32,
        neg_direction: i32,
    ) -> i32 {
        let mut amp: u8 = 0; // (re-)init, gets set by the measurement below
        let mut phs: u8 = 0; // (re-)init, gets set by the measurement below
        let mut f: u16;
        let mut bestdir = 0; /* Negative direction: decrease, Positive: increase. (-)1: aat_a, (-)2: aat_b */

        for i in -2..=2 {
            let mut a_test = ts.aat_a;
            let mut b_test = ts.aat_b;

            info!(
                "aat_steepest_desc dir i:{}, -dir: {}, -negdir: {}",
                i, -direction, -neg_direction
            );

            if (0 == i) || (i == -direction) || (i == -neg_direction) {
                /* Skip no direction and avoid going backwards */
                continue;
            }

            info!("Before step_dac_vals");
            let Ok(()) = self.aat_step_dac_vals(tp, &mut a_test, &mut b_test, i) else {
                info!("step_dac_vals returned err");
                continue; // Err returned: step_dac did nothing, try next direction
            };
            info!("After step_dac before aat_measure");
            self.aat_measure(&a_test, &b_test, &mut amp, &mut phs, ts).await;
            f = self.aat_calc_f(tp, amp, phs);

            info!(
                "Steepest descent result: i: {}, a_test: {}, b_test: {}, amp: {}, phs: {}, f: {}",
                i, a_test, b_test, amp, phs, f
            );

            if f < *f_min {
                /* Value is better than all previous ones */
                info!("**Better value!**, dir: {}", i);
                *f_min = f;
                bestdir = i;
            }
        }

        if 0 != bestdir {
            /* Walk into the best direction */
            let _ = self.aat_step_dac_vals(tp, &mut ts.aat_a, &mut ts.aat_b, bestdir);
        }
        bestdir
    }

    async fn aat_greedy_descent(
        &mut self,
        f_min: &mut u16,
        tp: &AatConfig,
        ts: &mut St25r3916AatTuneResult,
        direction: i32,
    ) -> i32 {
        let mut amp = 0; // (re-)init, gets set by the measurement below
        let mut phs = 0; // (re-)init, gets set by the measurement below
        let f;
        let mut a_test = ts.aat_a;
        let mut b_test = ts.aat_b;

        let Ok(()) = self.aat_step_dac_vals(tp, &mut a_test, &mut b_test, direction) else {
            info!("step_dac_vals returned err from greedy descent");
            return 0; // Err returned: step_dac did nothing, return 0
        };
        self.aat_measure(&a_test, &b_test, &mut amp, &mut phs, ts).await;
        f = self.aat_calc_f(tp, amp, phs);

        info!("Greedy descent result: ts.aat_a: {}, ts.aat_b: {}, f: {}", a_test, b_test, f);

        if f < *f_min {
            /* Value is better than previous one */
            info!("** Better value!**");
            ts.aat_a = a_test;
            ts.aat_b = b_test;
            *f_min = f;
            return direction;
        }

        0
    }

    fn aat_step_dac_vals(&mut self, tp: &AatConfig, a: &mut u8, b: &mut u8, dir: i32) -> Result<(), AntennaTuningError> {
        let mut a_changing = a.clone();
        let mut b_changing = b.clone();

        info!("start a & b: {} {}", a, b);

        match dir.abs() {
            /* Advance by steps size in requested direction */
            1 => {
                info!("step_dac 1");
                a_changing = if dir < 0 { *a - tp.a_step } else { *a + tp.a_step };

                a_changing = a_changing.clamp(tp.a_min, tp.a_max);
                if a_changing == *a {
                    return Err(AntennaTuningError::NoChange);
                }
            }
            2 => {
                info!("step_dac 2");
                b_changing = if dir < 0 { *b - tp.b_step } else { *b + tp.b_step };

                b_changing = b_changing.clamp(tp.b_min, tp.b_max);
                if b_changing == *b {
                    return Err(AntennaTuningError::NoChange);
                }
            }
            _ => {
                return Err(AntennaTuningError::InvalidDirection);
            }
        }

        *a = a_changing;
        *b = b_changing;

        info!("final a & b: {} {}", a, b);

        Ok(())
    }

    async fn aat_measure(&mut self, a: &u8, b: &u8, amp: &mut u8, phs: &mut u8, ts: &mut St25r3916AatTuneResult) {
        *amp = 0; // (re-)init, gets set by the measurement below
        *phs = 0; // (re-)init, gets set by the measurement below

        info!("Writing ant a and ant b registers with values: {} and {}", a, b);

        self.regs().ant_tune_a().write_value(*a).unwrap();
        self.regs().ant_tune_b().write_value(*b).unwrap();

        /* Wait till caps have settled.. */
        Timer::after(Duration::from_millis(ST25R3916_AAT_CAP_DELAY_MAX)).await;

        /* Get amplitude and phase .. */
        self.rfal_chip_measure_amplitude(amp).await;
        *phs = self.measure_phase().await.unwrap();

        /*
        FIXME: remove the ant reg reads below when AAT works. For debugging
        */
        let ant_a_value = self.regs().ant_tune_a().read().unwrap();
        let ant_b_value = self.regs().ant_tune_b().read().unwrap();
        info!("Registers AAT A and B after setting them: {} {}", ant_a_value, ant_b_value);

        info!("Measured amp and phase: {} and {}", amp, phs);

        ts.measureCnt += 1;
    }

    /*
    Amplitude measurement steps according to ST C lib
    */

    async fn rfal_chip_measure_amplitude(&mut self, amp: &mut u8) {
        /* Save registers which will be adjusted below */
        let reg_opc = self.regs().op_control().read().unwrap();
        let reg_mode = self.regs().mode().read().unwrap();
        let reg_conf1 = self.regs().rx_conf1().read().unwrap();
        let reg_conf2 = self.regs().rx_conf2().read().unwrap();
        let reg_auxmod = self.regs().aux_mod().read().unwrap();

        /* Set values as per defaults of DS. These regs/bits influence receiver chain and change amplitude */
        /* Doing so achieves an amplitude comparable over a complete polling cylce */
        self.regs()
            .op_control()
            .write_value(OpControl(reg_opc.0 & !ST25R3916_REG_OP_CONTROL_rx_chn))
            .unwrap();
        self.regs()
            .mode()
            .write_value(Mode(
                ST25R3916_REG_MODE_om_iso14443a
                    | ST25R3916_REG_MODE_targ_init
                    | ST25R3916_REG_MODE_tr_am_ook
                    | ST25R3916_REG_MODE_nfc_ar_off,
            ))
            .unwrap();
        self.regs()
            .rx_conf1()
            .write_value(RxConf1(reg_conf1.0 & !ST25R3916_REG_RX_CONF1_ch_sel_AM))
            .unwrap();
        self.regs()
            .rx_conf2()
            .write_value(RxConf2(
                reg_conf2.0
                    & !(ST25R3916_REG_RX_CONF2_demod_mode
                        | ST25R3916_REG_RX_CONF2_amd_sel
                        | ST25R3916_REG_RX_CONF2_amd_sel_peak),
            ))
            .unwrap();

        /* For ST25R3916B only */
        self.regs()
            .aux_mod()
            .write_value(AuxMod(reg_auxmod.0 & !ST25R3916_REG_AUX_MOD_rgs_am))
            .unwrap();

        /* Perform the actual measurement */
        *amp = self.measure_amplitude().await.unwrap();

        /* Restore values */
        self.regs().op_control().write_value(reg_opc).unwrap();
        self.regs().mode().write_value(reg_mode).unwrap();
        self.regs().rx_conf1().write_value(reg_conf1).unwrap();
        self.regs().rx_conf2().write_value(reg_conf2).unwrap();

        /* For ST25R3916B only */
        self.regs().aux_mod().write_value(reg_auxmod).unwrap();
    }

    fn aat_calc_f(&mut self, tp: &AatConfig, amp: u8, pha: u8) -> u16 {
        /* f(amp, pha) = (ampWeight * |amp - ampTarget|) + (phaWeight * |pha - phaTarget|) */
        let amp_delta = amp.abs_diff(tp.amp_target) as u16;
        let pha_delta = pha.abs_diff(tp.pha_target) as u16;

        let f = (tp.amp_weight * amp_delta) + (tp.pha_weight * pha_delta);
        f
    }
}
