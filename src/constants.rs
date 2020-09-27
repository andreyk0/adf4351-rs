//! Constants

/// Minimum allowed REFin frequency
pub const REF_IN_FREQ_MIN: u32 = 10_000_000;

/// Maximum allowed REFin frequency
pub const REF_IN_FREQ_MAX: u32 = 250_000_000;

/// Max Phase Detector Frequency (fractional N mode)
pub const PFD_FREQ_FRACN_MAX: u32 = 32_000_000;

/// The prescaler limits the INT value as follows:
/// Prescaler = 4/5: N MIN = 23
/// Prescaler = 8/9: N MIN = 75
/// This frequency is low enough that it covers VCO
/// range when multiplied by 75
pub const PFD_FREQ_FRACN_EASY_MAX: u32 = 29_000_000;

/// Max Phase Detector Frequency (Integer-N, band select enabled)
pub const PFD_FREQ_INTN_BS_MAX: u32 = 45_000_000;

/// Max Phase Detector Frequency (Integer-N, band select disabled)
pub const PFD_FREQ_INTN_MAX: u32 = 90_000_000;

/// Fundamental VCO mode (before dividers), min frequency
pub const VCO_FREQ_MIN: u64 = 2_200_000_000;

/// Fundamental VCO mode (before dividers), max frequency
pub const VCO_FREQ_MAX: u64 = 4_400_000_000;

/// Minimum allowed output frequency
/// 2200 MHz fundamental output and divide-by-64 selected
pub const OUT_FREQ_MIN: u64 = VCO_FREQ_MIN / 64;

/// VCO output, no divider
pub const OUT_FREQ_MAX: u64 = VCO_FREQ_MAX;

/// When the prescaler is set to
/// 4/5, the maximum RF frequency allowed is 3.6 GHz. Therefore,
/// when operating the ADF4351 above 3.6 GHz, the prescaler must
/// be set to 8/9.
pub const OUT_FREQ_P45_MAX: u64 = 3_600_000_000;
