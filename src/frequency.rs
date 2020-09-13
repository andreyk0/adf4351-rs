///! Frequency calculations

use crate::{constants::*, errors::*, register::*};


impl RegisterSet {

    /// New register set for a given frequency.
    /// TODO: optimal settings for different use cases
    pub fn newf(ref_in_hz: u32, out_hz: u64) -> Result<Self, Error> {
        (if !(REF_IN_FREQ_MIN .. REF_IN_FREQ_MAX).contains(&ref_in_hz) { Err(Error::InvalidReferenceFrequency) } else { Ok(())} ) ?;
        (if !(OUT_FREQ_MIN .. OUT_FREQ_MAX).contains(&out_hz) { Err(Error::InvalidOutputFrequency) } else { Ok(())} ) ?;

        let prescaler : Pr1Prescaler  =
            if out_hz > OUT_FREQ_P45_MAX {
                Pr1Prescaler::Pr89
            } else {
                Pr1Prescaler::Pr45
            };

        let mut vcof = out_hz;
        let mut rf_divider_select = 0;
        while vcof < VCO_FREQ_MIN { vcof *= 2; rf_divider_select += 1; }

        // configure fractional N mode
        // f PFD = REF IN × [(1 + D)/(R × (1 + T))]
        let (d,r,t) =
            if ref_in_hz > PFD_FREQ_FRACN_EASY_MAX {
                (RefDoubler::Enabled, 2 * ref_in_hz / PFD_FREQ_FRACN_EASY_MAX, Rdiv2::Enabled)
            } else {
                (RefDoubler::Enabled, 1, Rdiv2::Enabled) // normalize duty cycle
            };

        let fpfd = ref_in_hz * (1 + (d as u32)) / r / (1 + (t as u32));

        let modulus = 4000;

        // RF OUT = [INT + (FRAC/MOD)] × (f PFD /RF Divider)
        // RF_OUT * RF Divider / f_PFD = INT + FRAC/MOD
        let nscaled = (vcof * modulus) / fpfd as u64;
        let n = nscaled / modulus;
        let frac = nscaled % modulus;

        let r0: Reg0 = Reg0 {
            int: n as u16,
            frac: frac as u16
        };
        let r1: Reg1 = Reg1 {
            phase_adj: Ph1PhaseAdj::Off,
            prescaler,
            phase: 0,
            modulus: modulus as u16
        };
        let r2: Reg2 = Reg2 {
            noise_mode: NoiseMode::LowNoise,
            muxout: Muxout::ThreeStateOut,
            ref_doubler: d,
            rdiv2: t,
            r_counter: r as u16,
            double_buffer: DoubleBuffer::Enabled,
            cp_current: 0b111,
            ldf: Ldf::FracN,
            ldp: Ldp::Ldp10ns,
            pd_polarity: PdPolarity::Positive,
            power_down: PowerDown::Disabled,
            charge_pump: ChargePumpThreeState::Disabled,
            counter_reset: CounterReset::Disabled,
        };
        let r3: Reg3 = Reg3 {
            band_select_clock_mode: BandSelectClockMode::Low,
            anti_backlash_pulse_width: AntiBacklashPulseWidth::AB6ns,
            charge_cancellation: ChargeCancellation::Disabled,
            csr: CycleSlipReduction::Disabled,
            clock_divider_mode: ClockDividerMode::Off,
            clock_divider: 150,
        };
        let r4: Reg4 = Reg4 {
            feedback_select: FeedbackSelect::Fundamental,
            rf_divider_select,
            band_select_clock_div: 200,
            vco_power_down: VcoPowerDown::PoweredUp,
            mute_till_lock_detect: MuteTillLockDetect::Disabled,
            aux_output_select: AuxOutputSelect::Divided,
            aux_output_enable: AuxOutputEnable::Enabled,
            aux_output_power: 0b01,
            rf_output_enable: RfOutputEnable::Enabled,
            output_power: 0b01,
        };
        let r5: Reg5 = Reg5 {
            lock_detect_pin: LockDetectPin::DigitalLockDetect,
        };

        Ok(RegisterSet {r0, r1, r2, r3, r4, r5,})
    }

    /// Phase Frequency Detector' frequency
    /// f PFD = REF IN × [(1 + D)/(R × (1 + T))]
    /// where:
    /// REF IN is the reference frequency input.
    /// D is the RF REF IN doubler bit (0 or 1).
    /// R is the RF reference division factor (1 to 1023).
    /// T is the reference divide-by-2 bit (0 or 1).
    pub fn f_pfd(self: &Self, ref_in_hz: u32) -> f32 {
        (ref_in_hz as f32)
            * ( (1 + self.r2.ref_doubler as u32) as f32 )
            / ( (1 + self.r2.rdiv2 as u16) as f32 )
            / ( self.r2.r_counter as f32 )
    }

    /// Output frequency
    /// RF OUT = [INT + (FRAC/MOD)] × (f PFD /RF Divider)
    ///
    /// where:
    /// RF OUT is the RF frequency output.
    /// INT is the integer division factor.
    /// FRAC is the numerator of the fractional division (0 to MOD − 1).
    /// MOD is the preset fractional modulus (2 to 4095).
    /// RF Divider is the output divider that divides down the
    /// VCO frequency.
    pub fn f_out(self: &Self, ref_in_hz: u32) -> f32 {
        ( (self.r0.int as f32) +
          ( (self.r0.frac as f32) / (self.r1.modulus as f32) )
        ) * self.f_pfd(ref_in_hz)
          / ((1 << self.r4.rf_divider_select) as f32)
    }
}
