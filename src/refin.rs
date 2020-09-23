///! Input reference config
///! RefIn / Doubler / R counter / Divider

use crate::{ constants::*, errors::* };


/// Input reference frequency config
pub struct RefIn {
    /// Input frequency
    f: u32,
    /// R counter value
    r: u32,
    /// True if 2X doubler is enabled
    doubler: bool,
    /// True if 2X divider is enabled
    divider: bool,
}

impl RefIn {

    /// Configure reference input frequency
    pub fn new(
        f: u32,
        r: u32,
        doubler: bool,
        divider: bool,
    ) -> Result<Self, Error> {
        (if !(REF_IN_FREQ_MIN .. REF_IN_FREQ_MAX).contains(&f) { Err(Error::InvalidReferenceFrequency) } else { Ok(())} )?;

        let res = RefIn { f, r, doubler, divider };
        if res.f_pfd() > PFD_FREQ_INTN_MAX {
            // NOTE this is an absolute max, in FRAC-N mode the limit is even lower, just a sanity check
            Err(Error::InvalidReferenceFrequency)
        } else {
            Ok(res)
        }
    }

    /// Phase Frequency Detector' frequency
    /// f PFD = REF IN × [(1 + D)/(R × (1 + T))]
    /// where:
    /// REF IN is the reference frequency input.
    /// D is the RF REF IN doubler bit (0 or 1).
    /// R is the RF reference division factor (1 to 1023).
    /// T is the reference divide-by-2 bit (0 or 1).
    pub fn f_pfd(self: &Self) -> u32 {
        self.f
            * (1 + self.doubler as u32)
            / self.r
            / (1 + self.divider as u32)
    }
}
