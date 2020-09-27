//! ADF4351 registers

use core::marker::PhantomData;

/// Register number marker types
macro_rules! gen_register_marker {
    ($r:ident, $n:tt) => {
        /// Register $r maker
        #[derive(Debug,Copy,Clone)]
        pub struct $r {}

        impl Default for Reg<$r> { #[inline] fn default() -> Self { Reg { w: $n, phantom: PhantomData::default() } } }
    }
}

gen_register_marker!(R0, 0);
gen_register_marker!(R1, 1);
gen_register_marker!(R2, 2);
gen_register_marker!(R3, 3);
gen_register_marker!(R4, 4);
gen_register_marker!(R5, 5);


/// Single config register
#[derive(Debug,Copy,Clone)]
pub struct Reg<R> {
    /// Config register word
    pub w: u32,
    phantom: PhantomData<R>,
}

/// Bit operations on 32bit words
impl<R> Reg<R> {
    #[inline]
    pub fn get<F>(self: &Self) -> F
    where F: Sized + BitField<R> + From<u32>
    {
        F::from(
            (self.w >> F::offset()) & F::mask()
        )
    }

    #[inline]
    pub fn set<F>(mut self: Self, f: F) -> Self
    where F: Sized + BitField<R> + Into<u32>
    {
        let fbits = (f.into() & F::mask()) << F::offset();
        let rbits = self.w & (! ( F::mask() << F::offset() ));
        self.w = rbits | fbits;
        self
    }
}



/// Full set of config registers.
/// Defaults to all config bits set to 0.
///
/// When power is first applied to the ADF4351, the part requires
/// six writes (one each to R5, R4, R3, R2, R1, and R0) for the output
/// to become active.
#[derive(Debug,Copy,Clone,Default)]
pub struct RegisterSet {
    pub r0: Reg<R0>,
    pub r1: Reg<R1>,
    pub r2: Reg<R2>,
    pub r3: Reg<R3>,
    pub r4: Reg<R4>,
    pub r5: Reg<R5>,
}

/// Type-indexed register access
pub trait RIdx<R> {
    fn r(self: Self) -> Reg<R>;
    fn update_r<F>(self: Self, f: F) -> Self where F: FnOnce(Reg<R>) -> Reg<R>;
}

macro_rules! gen_register_index {
    ($r:ident, $f:tt) => {
        impl RIdx<$r> for RegisterSet {
            #[inline]
            fn r(self: Self) -> Reg<$r> { self.$f}
            #[inline]
            fn update_r<F>(mut self: Self, f: F) -> Self where F: FnOnce(Reg<$r>) -> Reg<$r> {
                self.$f = f(self.$f);
                self
            }
        }
    }
}

gen_register_index!(R0, r0);
gen_register_index!(R1, r1);
gen_register_index!(R2, r2);
gen_register_index!(R3, r3);
gen_register_index!(R4, r4);
gen_register_index!(R5, r5);


impl RegisterSet {

    /// Register values in device format.
    #[inline]
    pub fn to_words(self: &Self) -> &[u32; 6] {
        unsafe{ core::mem::transmute::<&RegisterSet, &[u32;6]>(&self) }
    }

    /// Get register bitfield value
    #[inline]
    pub fn get<F,R>(self: &Self) -> F
    where F: Sized + BitField<R> + From<u32>,
          Self: RIdx<R>
    {
        F::from(
            (self.r().w >> F::offset()) & F::mask()
        )
    }

    /// Update register bitfield
    #[inline]
    pub fn set<F,R>(self: Self, f: F) -> Self
    where F: Sized + BitField<R> + Into<u32>,
          Self: RIdx<R>
    {
        self.update_r(|r| r.set(f))
    }
}



/// Bit operations on 32bit words
pub trait BitField<R> {
    /// Number of bits in the bit field
    fn num_bits() -> u8;

    /// Offset from 0
    fn offset() -> u8;

    #[inline]
    fn mask() -> u32 {
        !(0xFFFFFFFFu32 << Self::num_bits())
    }
}

/// Generate BitField implementation
macro_rules! gen_bitfield_impl {
	($r:ty, $n:ident, $nb:tt, $off:tt) => {
        impl BitField<$r> for $n {
            #[inline] fn num_bits() -> u8 { $nb }
            #[inline] fn offset() -> u8 { $off }
        }
    }
}

/// Small bitfield-encoded numbes boilerplate
macro_rules! gen_bitfield_struct {
	($(#[$meta:meta])*, $r:ty, $n:ident, $v:ty, $nb:tt, $off:tt) => {
        $(#[$meta])*
        #[derive(Debug,Copy,Clone)]
        pub struct $n(pub $v);

        gen_bitfield_impl!($r, $n, $nb, $off);

        impl From<u32> for $n { #[inline] fn from(x: u32) -> Self { $n(x as $v) } }
        impl Into<u32> for $n { #[inline] fn into(self) -> u32 { self.0 as u32 } }
	};
}

macro_rules! gen_bitfield_enum {
	($r:ty, $n:ident, $nb:tt, $off:tt) => {
        gen_bitfield_impl!($r, $n, $nb, $off);

        impl From<u32> for $n { #[inline] fn from(x: u32) -> Self { x.into() } }
        impl Into<u32> for $n { #[inline] fn into(self) -> u32 { self as u32 } }
    }
}


gen_bitfield_struct!(
    /// The 16 INT bits (Bits[DB30:DB15]) set the INT value, which
    /// determines the integer part of the feedback division factor. The
    /// INT value is used in Equation 1 (see the INT, FRAC, MOD, and
    /// R Counter Relationship section). Integer values from 23 to
    /// 65,535 are allowed for the 4/5 prescaler; for the 8/9 prescaler,
    /// the minimum integer value is 75.
    , R0, Int, u16, 16, 15
);


gen_bitfield_struct!(
    ///  The 12 FRAC bits (Bits[DB14:DB3]) set the numerator of the
    ///  fraction that is input to the Σ-Δ modulator. This fraction, along
    ///  with the INT value, specifies the new frequency channel that
    ///  the synthesizer locks to, as shown in the RF Synthesizer—A
    ///  Worked Example section. FRAC values from 0 to (MOD − 1)
    ///  cover channels over a frequency range equal to the PFD refer-
    ///  ence frequency.
    , R0, Frac, u16, 12, 3
);


/// The phase adjust bit (Bit DB28) enables adjustment of the output
/// phase of a given output frequency. When phase adjustment is
/// enabled (Bit DB28 is set to 1), the part does not perform VCO
/// band selection or phase resync when Register 0 is updated.
/// When phase adjustment is disabled (Bit DB28 is set to 0), the
/// part performs VCO band selection and phase resync (if phase
/// resync is enabled in Register 3, Bits[DB16:DB15]) when Register 0
/// is updated. Disabling VCO band selection is recommended only
/// for fixed frequency applications or for frequency deviations of
/// <1 MHz from the originally selected frequency.
#[derive(Debug,Copy,Clone)]
pub enum Ph1PhaseAdj {
    Off,
    On,
}
gen_bitfield_enum!(R1, Ph1PhaseAdj, 1, 28);




/// The dual-modulus prescaler (P/P + 1), along with the INT,
/// FRAC, and MOD values, determines the overall division
/// ratio from the VCO output to the PFD input. The PR1 bit
/// (DB27) in Register 1 sets the prescaler value.
/// Operating at CML levels, the prescaler takes the clock from the
/// VCO output and divides it down for the counters. The prescaler
/// is based on a synchronous 4/5 core. When the prescaler is set to
/// 4/5, the maximum RF frequency allowed is 3.6 GHz. Therefore,
/// when operating the ADF4351 above 3.6 GHz, the prescaler must
/// be set to 8/9.
#[derive(Debug,Copy,Clone)]
pub enum Pr1Prescaler {
    /// Prescaler = 4/5: INT N MIN = 23
    Pr45,
    /// Prescaler = 8/9: INT N MIN = 75
    Pr89,
}
gen_bitfield_enum!(R1, Pr1Prescaler, 1, 27);


gen_bitfield_struct!(
    /// 12-Bit Phase Value
    /// Bits[DB26:DB15] control the phase word. The phase word must
    /// be less than the MOD value programmed in Register 1. The phase
    /// word is used to program the RF output phase from 0° to 360°
    /// with a resolution of 360°/MOD (see the Phase Resync section).
    , R1, Phase, u16, 12, 15
);


gen_bitfield_struct!(
    /// 12-Bit Modulus Value (MOD)
    /// The 12 MOD bits (Bits[DB14:DB3]) set the fractional modulus.
    /// The fractional modulus is the ratio of the PFD frequency to the
    /// channel step resolution on the RF output. For more information,
    /// see the 12-Bit Programmable Modulus section.
    , R1, Mod, u16, 12, 3
);




/// The noise mode on the ADF4351 is controlled by setting
/// Bits[DB30:DB29] in Register 2 (see Figure 26). The noise mode
/// allows the user to optimize a design either for improved spurious
/// performance or for improved phase noise performance.
/// When the low spur mode is selected, dither is enabled. Dither
/// randomizes the fractional quantization noise so that it resembles
/// white noise rather than spurious noise. As a result, the part is
/// optimized for improved spurious performance. Low spur mode
/// is normally used for fast-locking applications when the PLL
/// closed-loop bandwidth is wide. Wide loop bandwidth is a loop
/// bandwidth greater than 1/10 of the RF OUT channel step resolu-
/// tion (f RES ). A wide loop filter does not attenuate the spurs to the
/// same level as a narrow loop bandwidth.
/// For best noise performance, use the low noise mode option.
/// When the low noise mode is selected, dither is disabled. This
/// mode ensures that the charge pump operates in an optimum
/// region for noise performance. Low noise mode is extremely
/// useful when a narrow loop filter bandwidth is available. The
/// synthesizer ensures extremely low noise, and the filter attenuates
/// the spurs. Figure 10 through Figure 12 show the trade-offs in a
/// typical W-CDMA setup for different noise and spur settings.
#[derive(Debug,Copy,Clone)]
pub enum NoiseMode {
    LowNoise,
    LowSpur = 0b11,
}
gen_bitfield_enum!(R2, NoiseMode, 2, 29);


/// The on-chip multiplexer is controlled by Bits[DB28:DB26]
/// (see Figure 26). Note that N counter output must be disabled
/// for VCO band selection to operate correctly.
#[derive(Debug,Copy,Clone)]
pub enum Muxout {
    ThreeStateOut,
    Dvdd,
    Dgnd,
    RCntOut,
    NDivOut,
    Alock,
    Dlock,
}
gen_bitfield_enum!(R2, Muxout, 3, 26);


/// Setting the DB25 bit to 0 disables the doubler and feeds the REF IN
/// signal directly into the 10-bit R counter. Setting this bit to 1 multi-
/// plies the REF IN frequency by a factor of 2 before feeding it into
/// the 10-bit R counter. When the doubler is disabled, the REF IN
/// falling edge is the active edge at the PFD input to the fractional
/// synthesizer. When the doubler is enabled, both the rising and
/// falling edges of REF IN become active edges at the PFD input.
/// When the doubler is enabled and the low spur mode is selected,
/// the in-band phase noise performance is sensitive to the REF IN duty
/// cycle. The phase noise degradation can be as much as 5 dB for
/// REF IN duty cycles outside a 45% to 55% range. The phase noise
/// is insensitive to the REF IN duty cycle in the low noise mode and
/// when the doubler is disabled.
/// The maximum allowable REF IN frequency when the doubler is
/// enabled is 30 MHz.
#[derive(Debug,Copy,Clone)]
pub enum RefDoubler {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R2, RefDoubler, 1, 25);


/// Setting the DB24 bit to 1 inserts a divide-by-2 toggle flip-flop
/// between the R counter and the PFD, which extends the maximum
/// REF IN input rate. This function allows a 50% duty cycle signal to
/// appear at the PFD input, which is necessary for cycle slip reduction.
#[derive(Debug,Copy,Clone)]
pub enum Rdiv2 {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R2, Rdiv2, 1, 24);

gen_bitfield_struct!(
    /// The 10-bit R counter (Bits[DB23:DB14]) allows the input reference
    /// frequency (REF IN ) to be divided down to produce the reference
    /// clock to the PFD. Division ratios from 1 to 1023 are allowed.
    , R2, R, u16, 10, 14
);

/// The DB13 bit enables or disables double buffering of
/// Bits[DB22:DB20] in Register 4. For information about how
/// double buffering works, see the Program Modes section.
#[derive(Debug,Copy,Clone)]
pub enum DoubleBuffer {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R2, DoubleBuffer, 1, 13);


gen_bitfield_struct!(
    /// Charge Pump Current Setting
    /// Bits[DB12:DB9] set the charge pump current. This value should
    /// be set to the charge pump current that the loop filter is designed
    /// with (see Figure 26).
    , R2, ChargePumpCurrent, u8, 4, 9
);


/// The DB8 bit configures the lock detect function (LDF). The LDF
/// controls the number of PFD cycles monitored by the lock detect
/// circuit to ascertain whether lock has been achieved. When DB8 is
/// set to 0, the number of PFD cycles monitored is 40. When DB8
/// is set to 1, the number of PFD cycles monitored is 5. It is recom-
/// mended that the DB8 bit be set to 0 for fractional-N mode and
/// to 1 for integer-N mode.
#[derive(Debug,Copy,Clone)]
pub enum Ldf {
    FracN,
    IntN,
}
gen_bitfield_enum!(R2, Ldf, 1, 8);


/// The lock detect precision bit (Bit DB7) sets the comparison
/// window in the lock detect circuit. When DB7 is set to 0, the
/// comparison window is 10 ns; when DB7 is set to 1, the window
/// is 6 ns. The lock detect circuit goes high when n consecutive
/// PFD cycles are less than the comparison window value; n is set
/// by the LDF bit (DB8). For example, with DB8 = 0 and DB7 = 0,
/// 40 consecutive PFD cycles of 10 ns or less must occur before
/// digital lock detect goes high.
/// For fractional-N applications, the recommended setting for
/// Bits[DB8:DB7] is 00; for integer-N applications, the recom-
/// mended setting for Bits[DB8:DB7] is 11.
#[derive(Debug,Copy,Clone)]
pub enum Ldp {
    Ldp10ns,
    Ldp6ns,
}
gen_bitfield_enum!(R2, Ldp, 1, 7);


/// The DB6 bit sets the phase detector polarity. When a passive
/// loop filter or a noninverting active loop filter is used, this bit
/// should be set to 1. If an active filter with an inverting charac-
/// teristic is used, this bit should be set to 0.
#[derive(Debug,Copy,Clone)]
pub enum PhaseDetectorPolarity {
    Negative,
    Positive,
}
gen_bitfield_enum!(R2, PhaseDetectorPolarity, 1, 6);


/// The DB5 bit provides the programmable power-down mode.
/// Setting this bit to 1 performs a power-down. Setting this bit to 0
/// returns the synthesizer to normal operation. In software power-
/// down mode, the part retains all information in its registers. The
/// register contents are lost only if the supply voltages are removed.
/// When power-down is activated, the following events occur:
///
/// * Synthesizer counters are forced to their load state conditions.
/// * VCO is powered down.
/// * Charge pump is forced into three-state mode.
/// * Digital lock detect circuitry is reset.
/// * RF OUT buffers are disabled.
/// * Input registers remain active and capable of loading and latching data.
#[derive(Debug,Copy,Clone)]
pub enum PowerDown {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R2, PowerDown, 1, 5);


/// Setting the DB4 bit to 1 puts the charge pump into three-state
/// mode. This bit should be set to 0 for normal operation.
#[derive(Debug,Copy,Clone)]
pub enum ChargePumpThreeState {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R2, ChargePumpThreeState, 1, 4);


/// The DB3 bit is the reset bit for the R counter and the N counter
/// of the ADF4351. When this bit is set to 1, the RF synthesizer
/// N counter and R counter are held in reset. For normal opera-
/// tion, this bit should be set to 0.
#[derive(Debug,Copy,Clone)]
pub enum CounterReset {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R2, CounterReset, 1, 3);




/// Setting the DB23 bit to 1 selects a faster logic sequence of band
/// selection, which is suitable for high PFD frequencies and is
/// necessary for fast lock applications. Setting the DB23 bit to 0 is
/// recommended for low PFD (<125 kHz) values. For the faster
/// band select logic modes (DB23 set to 1), the value of the band
/// select clock divider must be less than or equal to 254.
#[derive(Debug,Copy,Clone)]
pub enum BandSelectClockMode {
    Low,
    High,
}
gen_bitfield_enum!(R3, BandSelectClockMode, 1, 23);


/// Bit DB22 sets the PFD antibacklash pulse width. When Bit DB22
/// is set to 0, the PFD antibacklash pulse width is 6 ns. This setting is
/// recommended for fractional-N use. When Bit DB22 is set to 1,
/// the PFD antibacklash pulse width is 3 ns, which results in phase
/// noise and spur improvements in integer-N operation. For
/// fractional-N operation, the 3 ns setting is not recommended.
#[derive(Debug,Copy,Clone)]
pub enum AntiBacklashPulseWidth {
    AB6ns, // FRAC-N
    AB3ns, // INT-N
}
gen_bitfield_enum!(R3, AntiBacklashPulseWidth, 1, 22);


/// Setting the DB21 bit to 1 enables charge pump charge cancel-
/// ation. This has the effect of reducing PFD spurs in integer-N
/// mode. In fractional-N mode, this bit should be set to 0.
#[derive(Debug,Copy,Clone)]
pub enum ChargeCancellation {
    Disabled, // FRAC-N
    Enabled, // INT-N
}
gen_bitfield_enum!(R3, ChargeCancellation, 1, 21);


/// Setting the DB18 bit to 1 enables cycle slip reduction. CSR is
/// a method for improving lock times. Note that the signal at the
/// phase frequency detector (PFD) must have a 50% duty cycle for
/// cycle slip reduction to work. The charge pump current setting
/// must also be set to a minimum. For more information, see the
/// Cycle Slip Reduction for Faster Lock Times section.
#[derive(Debug,Copy,Clone)]
pub enum CycleSlipReduction {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R3, CycleSlipReduction, 1, 18);


/// Bits[DB16:DB15] must be set to 10 to activate phase resync
/// (see the Phase Resync section). These bits must be set to 01
/// to activate fast lock (see the Fast Lock Timer and Register
/// Sequences section). Setting Bits[DB16:DB15] to 00 disables
/// the clock divider (see Figure 27).
#[derive(Debug,Copy,Clone)]
pub enum ClockDividerMode {
    Off,
    FastLock,
    Resync,
}
gen_bitfield_enum!(R3, ClockDividerMode, 2, 15);


gen_bitfield_struct!(
    /// 12-Bit Clock Divider Value
    /// Bits[DB14:DB3] set the 12-bit clock divider value. This value
    /// is the timeout counter for activation of phase resync (see the
    /// Phase Resync section). The clock divider value also sets the
    /// timeout counter for fast lock (see the Fast Lock Timer and
    /// Register Sequences section).
    , R3, ClockDividerValue, u16, 12, 3
);



/// The DB23 bit selects the feedback from the VCO output to the
/// N counter. When this bit is set to 1, the signal is taken directly
/// from the VCO. When this bit is set to 0, the signal is taken from
/// the output of the output dividers. The dividers enable coverage
/// of the wide frequency band (34.375 MHz to 4.4 GHz). When
/// the dividers are enabled and the feedback signal is taken from
/// the output, the RF output signals of two separately configured
/// PLLs are in phase. This is useful in some applications where the
/// positive interference of signals is required to increase the power.
#[derive(Debug,Copy,Clone)]
pub enum FeedbackSelect {
    Divided,
    Fundamental,
}
gen_bitfield_enum!(R4, FeedbackSelect, 1, 23);


gen_bitfield_struct!(
    /// RF Divider Select
    /// Bits[DB22:DB20] select the value of the RF output divider (see
    /// Figure 28).
    , R4, RfDividerSelect, u8, 3, 20
);


gen_bitfield_struct!(
    /// Band Select Clock Divider Value
    /// Bits[DB19:DB12] set a divider for the band select logic clock input.
    /// By default, the output of the R counter is the value used to clock
    /// the band select logic, but, if this value is too high (>125 kHz), a
    /// divider can be switched on to divide the R counter output to a
    /// smaller value (see Figure 28).
    , R4, BandSelectClockDiv, u8, 8, 12
);


/// Setting the DB11 bit to 0 powers the VCO up; setting this bit to 1
/// powers the VCO down.
#[derive(Debug,Copy,Clone)]
pub enum VcoPowerDown {
    PoweredUp,
    PoweredDown,
}
gen_bitfield_enum!(R4, VcoPowerDown, 1, 11);


/// When the DB10 bit is set to 1, the supply current to the RF output
/// stage is shut down until the part achieves lock, as measured by
/// the digital lock detect circuitry.
#[derive(Debug,Copy,Clone)]
pub enum MuteTillLockDetect {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R4, MuteTillLockDetect, 1, 10);


/// The DB9 bit sets the auxiliary RF output. If DB9 is set to 0, the
/// auxiliary RF output is the output of the RF dividers; if DB9 is set
/// to 1, the auxiliary RF output is the fundamental VCO frequency.
#[derive(Debug,Copy,Clone)]
pub enum AuxOutputSelect {
    Divided,
    Fundamental,
}
gen_bitfield_enum!(R4, AuxOutputSelect, 1, 9);


/// The DB8 bit enables or disables the auxiliary RF output. If DB8
/// is set to 0, the auxiliary RF output is disabled; if DB8 is set to 1,
/// the auxiliary RF output is enabled.
#[derive(Debug,Copy,Clone)]
pub enum AuxOutputEnable {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R4, AuxOutputEnable, 1, 8);

gen_bitfield_struct!(
    /// AUX Output Power
    /// Bits[DB7:DB6] set the value of the auxiliary RF output power
    /// level (see Figure 28).
    , R4, AuxOutputPower, u8, 2, 6
);


/// The DB5 bit enables or disables the primary RF output. If DB5
/// is set to 0, the primary RF output is disabled; if DB5 is set to 1,
/// the primary RF output is enabled.
#[derive(Debug,Copy,Clone)]
pub enum RfOutputEnable {
    Disabled,
    Enabled,
}
gen_bitfield_enum!(R4, RfOutputEnable, 1, 5);


gen_bitfield_struct!(
    /// Output Power
    /// Bits[DB4:DB3] set the value of the primary RF output power
    /// level (see Figure 28).
    , R4, OutputPower, u8, 2, 3
);


/// Bits[DB23:DB22] set the operation of the lock detect (LD) pin
/// (see Figure 29).
#[derive(Debug,Copy,Clone)]
pub enum LockDetectPin {
    Low,
    DigitalLockDetect,
    Low1,
    High,
}
gen_bitfield_enum!(R5, LockDetectPin, 2, 22);
