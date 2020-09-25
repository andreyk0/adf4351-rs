///! Device configuration / frequency calculations

use crate::{ constants::*, errors::*,register::*};


/// Phase Frequency Detector' frequency, Hz
/// f PFD = REF IN × [(1 + D)/(R × (1 + T))]
#[derive(Debug,Copy,Clone)]
pub struct Fpfd(pub u32);

impl Fpfd {
    /// Calculate Phase Frequency Detector' frequency
    /// f PFD = REF IN × [(1 + D)/(R × (1 + T))]
    /// where:
    /// REF IN is the reference frequency input.
    /// D is the RF REF IN doubler bit (0 or 1).
    /// R is the RF reference division factor (1 to 1023).
    /// T is the reference divide-by-2 bit (0 or 1).
    pub fn new(
        ref_in_hz: u32,
        rs: &RegisterSet,
    ) -> Result<Self,Error> {
        (if !(REF_IN_FREQ_MIN .. REF_IN_FREQ_MAX).contains(&ref_in_hz) { Err(Error::InvalidReferenceFrequency) } else { Ok(())} )?;

        let doubler : RefDoubler = rs.get();
        let divider : Rdiv2 = rs.get();
        let r : R = rs.get();
        let fpfd = ref_in_hz * (1 + doubler as u32) / (r.0 as u32) / (1 + divider as u32);

        if fpfd > PFD_FREQ_INTN_MAX {
            // NOTE this is an absolute max, in FRAC-N mode the limit is even lower, just a sanity check
            Err(Error::InvalidReferenceFrequency)
        } else {
            Ok(Fpfd(fpfd))
        }
    }
}


/// FRAC-N frequency settings
#[derive(Debug,Copy,Clone)]
pub struct FracN(pub Fpfd);

impl FracN {

    /// Initialize FracN mode
    pub fn init(rs: RegisterSet) -> RegisterSet {
        rs.set(FeedbackSelect::Fundamental) // set_f_out calculation is based on Fundamental VCO feedback frequency
          .set(Ldf::FracN)
          .set(Ldp::Ldp10ns)
          .set(AntiBacklashPulseWidth::AB6ns)
    }


    /// Sets output frequency to the value close to the desired.
    /// Actual frequency will depend on the REF IN and modulus settings.
    pub fn set_f_out(
        self: &Self,
        f_out_hz: u64,
        rs: RegisterSet
    ) -> Result<RegisterSet, Error> {
        (if !(OUT_FREQ_MIN .. OUT_FREQ_MAX).contains(&f_out_hz) { Err(Error::InvalidOutputFrequency) } else { Ok(())} ) ?;

        let prescaler : Pr1Prescaler  =
            if f_out_hz > OUT_FREQ_P45_MAX {
                Pr1Prescaler::Pr89
            } else {
                Pr1Prescaler::Pr45
            };

        let mut vcof = f_out_hz;
        let mut rf_divider_select = 0;
        while vcof < VCO_FREQ_MIN { vcof *= 2; rf_divider_select += 1; }

        let rmod : Mod = rs.get();
        let modulus = rmod.0 as u64;

        // RF OUT = [INT + (FRAC/MOD)] × (f PFD /RF Divider)
        // RF_OUT * RF Divider / f_PFD = INT + FRAC/MOD
        let nscaled = (vcof * modulus) / self.0.0 as u64;
        let int = nscaled / modulus;
        let frac = nscaled % modulus;

        Ok (
            rs.set(Int(int as u16))
              .set(Frac(frac as u16))
              .set(RfDividerSelect(rf_divider_select))
              .set(prescaler)
        )
    }


    /// Calculate actual output frequency from current register values.
    /// RF OUT = [INT + (FRAC/MOD)] × (f PFD /RF Divider)
    ///
    /// where:
    /// RF OUT is the RF frequency output.
    /// INT is the integer division factor.
    /// FRAC is the numerator of the fractional division (0 to MOD − 1).
    /// MOD is the preset fractional modulus (2 to 4095).
    /// RF Divider is the output divider that divides down the
    /// VCO frequency.
    pub fn f_out_hz(ref_in_hz: u32, rs: &RegisterSet) -> Result<u64,Error> {
        let int : Int = rs.get();
        let int = int.0 as u64;

        let frac : Frac = rs.get();
        let frac = frac.0 as u64;

        let modulus : Mod = rs.get();
        let modulus = modulus.0 as u64;

        let rfdiv : RfDividerSelect = rs.get();
        let rfdiv : u64 = 1 << rfdiv.0;

        let fpfd = Fpfd::new(ref_in_hz, rs)?;
        let fpfd = fpfd.0 as u64;

        Ok(
            (int*fpfd + frac*fpfd/modulus) / rfdiv
        )
    }
}
