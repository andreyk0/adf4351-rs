///! ADF4351 registers

/// When power is first applied to the ADF4351, the part requires
/// six writes (one each to R5, R4, R3, R2, R1, and R0) for the output
/// to become active.
#[derive(Debug,Copy,Clone)]
pub struct RegisterSet {
    pub r0: Reg0,
    pub r1: Reg1,
    pub r2: Reg2,
    pub r3: Reg3,
    pub r4: Reg4,
    pub r5: Reg5,
}

impl RegisterSet {
    /// Register values in device format.
    /// This is the order in which they should be sent to device.
    pub fn to_words(self: &Self) -> [u32; 6] {
        let pr = self.r1.prescaler;
        [ self.r5.to_word(pr),
          self.r4.to_word(pr),
          self.r3.to_word(pr),
          self.r2.to_word(pr),
          self.r1.to_word(pr),
          self.r0.to_word(pr),
        ]
    }
}


/// Register conversion to u32, to be sent over SPI
pub trait Reg {
    fn to_word(self: &Self, pr1: Pr1Prescaler) -> u32;
}

/// Register 0
#[derive(Debug,Copy,Clone)]
pub struct Reg0 {
    /// The 16 INT bits (Bits[DB30:DB15]) set the INT value, which
    /// determines the integer part of the feedback division factor. The
    /// INT value is used in Equation 1 (see the INT, FRAC, MOD, and
    /// R Counter Relationship section). Integer values from 23 to
    /// 65,535 are allowed for the 4/5 prescaler; for the 8/9 prescaler,
    /// the minimum integer value is 75.
    pub int: u16,

    ///  The 12 FRAC bits (Bits[DB14:DB3]) set the numerator of the
    ///  fraction that is input to the Σ-Δ modulator. This fraction, along
    ///  with the INT value, specifies the new frequency channel that
    ///  the synthesizer locks to, as shown in the RF Synthesizer—A
    ///  Worked Example section. FRAC values from 0 to (MOD − 1)
    ///  cover channels over a frequency range equal to the PFD refer-
    ///  ence frequency.
    pub frac: u16, // lower 12 bits are used
}

impl Reg for Reg0 {
    fn to_word(self: &Self, pr1: Pr1Prescaler) -> u32 {
        let intmin = match pr1 {
            Pr1Prescaler::Pr45 => 23,
            Pr1Prescaler::Pr89 => 75,

        };
        (self.int.max(intmin) as u32) << 15 |
        ((self.frac & 0xFFF) as u32) << 3
    }
}


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

/// Register 1
#[derive(Debug,Copy,Clone)]
pub struct Reg1 {
    /// Phase Adjust
    pub phase_adj: Ph1PhaseAdj,

    ///  Prescaler Value
    pub prescaler: Pr1Prescaler,

    /// 12-Bit Phase Value
    /// Bits[DB26:DB15] control the phase word. The phase word must
    /// be less than the MOD value programmed in Register 1. The phase
    /// word is used to program the RF output phase from 0° to 360°
    /// with a resolution of 360°/MOD (see the Phase Resync section).
    pub phase: u16,

    /// 12-Bit Modulus Value (MOD)
    /// The 12 MOD bits (Bits[DB14:DB3]) set the fractional modulus.
    /// The fractional modulus is the ratio of the PFD frequency to the
    /// channel step resolution on the RF output. For more information,
    /// see the 12-Bit Programmable Modulus section.
    pub modulus: u16,

}

impl Reg for Reg1 {
    fn to_word(self: &Self, _pr1: Pr1Prescaler) -> u32 {
        (self.phase_adj as u32) << 28 |
        (self.prescaler  as u32) << 27 |
        ((self.phase as u32) & 0xFFF) << 15 |
        (((self.modulus & 0xFFF).max(2) as u32) & 0xFFF) << 3 |
        0b001
    }
}

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

/// Setting the DB24 bit to 1 inserts a divide-by-2 toggle flip-flop
/// between the R counter and the PFD, which extends the maximum
/// REF IN input rate. This function allows a 50% duty cycle signal to
/// appear at the PFD input, which is necessary for cycle slip reduction.
#[derive(Debug,Copy,Clone)]
pub enum Rdiv2 {
    Disabled,
    Enabled,
}

/// The DB13 bit enables or disables double buffering of
/// Bits[DB22:DB20] in Register 4. For information about how
/// double buffering works, see the Program Modes section.
#[derive(Debug,Copy,Clone)]
pub enum DoubleBuffer {
    Disabled,
    Enabled,
}

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


/// The DB6 bit sets the phase detector polarity. When a passive
/// loop filter or a noninverting active loop filter is used, this bit
/// should be set to 1. If an active filter with an inverting charac-
/// teristic is used, this bit should be set to 0.
#[derive(Debug,Copy,Clone)]
pub enum PdPolarity {
    Negative,
    Positive,
}

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

/// Setting the DB4 bit to 1 puts the charge pump into three-state
/// mode. This bit should be set to 0 for normal operation.
#[derive(Debug,Copy,Clone)]
pub enum ChargePumpThreeState {
    Disabled,
    Enabled,
}

/// The DB3 bit is the reset bit for the R counter and the N counter
/// of the ADF4351. When this bit is set to 1, the RF synthesizer
/// N counter and R counter are held in reset. For normal opera-
/// tion, this bit should be set to 0.
#[derive(Debug,Copy,Clone)]
pub enum CounterReset {
    Disabled,
    Enabled,
}

/// Register 2
#[derive(Debug,Copy,Clone)]
pub struct Reg2 {
    /// Low Noise and Low Spur Modes
    pub noise_mode: NoiseMode,

    /// MUXOUT
    pub muxout: Muxout,

    /// Reference Doubler
    pub ref_doubler: RefDoubler,

    /// RDIV2
    pub rdiv2: Rdiv2,

    /// The 10-bit R counter (Bits[DB23:DB14]) allows the input reference
    /// frequency (REF IN ) to be divided down to produce the reference
    /// clock to the PFD. Division ratios from 1 to 1023 are allowed.
    pub r_counter: u16, // 10bit

    /// Double Buffer
    pub double_buffer: DoubleBuffer,

    /// Charge Pump Current Setting
    /// Bits[DB12:DB9] set the charge pump current. This value should
    /// be set to the charge pump current that the loop filter is designed
    /// with (see Figure 26).
    pub cp_current: u8, // 4bit

    /// Lock Detect Function (LDF)
    pub ldf: Ldf,

    /// Lock Detect Precision (LDP)
    pub ldp: Ldp,

    /// Phase Detector Polarity
    pub pd_polarity: PdPolarity,

    /// Power-Down (PD)
    pub power_down: PowerDown,

    /// Charge Pump Three-State
    pub charge_pump: ChargePumpThreeState,

    /// Counter Reset
    pub counter_reset: CounterReset,
}

impl Reg for Reg2 {
    fn to_word(self: &Self, _pr1: Pr1Prescaler) -> u32 {
        (self.noise_mode as u32) << 29 |
        (self.muxout as u32) << 26 |
        (self.ref_doubler as u32) << 25 |
        (self.rdiv2 as u32) << 24 |
        (self.r_counter.max(1).min(1023) as u32) << 14 |
        (self.double_buffer as u32) << 13 |
        ((self.cp_current & 0x0F) as u32) << 9 |
        (self.ldf as u32) << 8 |
        (self.ldp as u32) << 7 |
        (self.pd_polarity as u32) << 6 |
        (self.power_down as u32) << 5 |
        (self.charge_pump as u32) << 4 |
        (self.counter_reset as u32) << 3 |
        0b010
    }
}

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

/// Setting the DB21 bit to 1 enables charge pump charge cancel-
/// ation. This has the effect of reducing PFD spurs in integer-N
/// mode. In fractional-N mode, this bit should be set to 0.
#[derive(Debug,Copy,Clone)]
pub enum ChargeCancellation {
    Disabled, // FRAC-N
    Enabled, // INT-N
}


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


/// Register 3
#[derive(Debug,Copy,Clone)]
pub struct Reg3 {
    /// Band Select Clock Mode
    pub band_select_clock_mode: BandSelectClockMode,

    /// Antibacklash Pulse Width (ABP)
    pub anti_backlash_pulse_width: AntiBacklashPulseWidth,

    /// Charge Cancelation
    pub charge_cancellation: ChargeCancellation,

    /// CSR Enable
    pub csr: CycleSlipReduction,

    /// Clock Divider Mode
    pub clock_divider_mode: ClockDividerMode,

    /// 12-Bit Clock Divider Value
    /// Bits[DB14:DB3] set the 12-bit clock divider value. This value
    /// is the timeout counter for activation of phase resync (see the
    /// Phase Resync section). The clock divider value also sets the
    /// timeout counter for fast lock (see the Fast Lock Timer and
    /// Register Sequences section).
    pub clock_divider: u16, // 12bit
}

impl Reg for Reg3 {
    fn to_word(self: &Self, _pr1: Pr1Prescaler) -> u32 {
        (self.band_select_clock_mode as u32) << 23 |
        (self.anti_backlash_pulse_width as u32) << 22 |
        (self.charge_cancellation as u32) << 21 |
        (self.csr as u32) << 18 |
        (self.clock_divider_mode as u32) << 15 |
        ((self.clock_divider & 0xFFF) as u32) << 3 |
        0b011
    }
}

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

/// Setting the DB11 bit to 0 powers the VCO up; setting this bit to 1
/// powers the VCO down.
#[derive(Debug,Copy,Clone)]
pub enum VcoPowerDown {
    PoweredUp,
    PoweredDown,
}

/// When the DB10 bit is set to 1, the supply current to the RF output
/// stage is shut down until the part achieves lock, as measured by
/// the digital lock detect circuitry.
#[derive(Debug,Copy,Clone)]
pub enum MuteTillLockDetect {
    Disabled,
    Enabled,
}

/// The DB9 bit sets the auxiliary RF output. If DB9 is set to 0, the
/// auxiliary RF output is the output of the RF dividers; if DB9 is set
/// to 1, the auxiliary RF output is the fundamental VCO frequency.
#[derive(Debug,Copy,Clone)]
pub enum AuxOutputSelect {
    Divided,
    Fundamental,
}

/// The DB8 bit enables or disables the auxiliary RF output. If DB8
/// is set to 0, the auxiliary RF output is disabled; if DB8 is set to 1,
/// the auxiliary RF output is enabled.
#[derive(Debug,Copy,Clone)]
pub enum AuxOutputEnable {
    Disabled,
    Enabled,
}

/// The DB5 bit enables or disables the primary RF output. If DB5
/// is set to 0, the primary RF output is disabled; if DB5 is set to 1,
/// the primary RF output is enabled.
#[derive(Debug,Copy,Clone)]
pub enum RfOutputEnable {
    Disabled,
    Enabled,
}

/// Register 4
#[derive(Debug,Copy,Clone)]
pub struct Reg4 {
    /// Feedback Select
    pub feedback_select: FeedbackSelect,

    /// RF Divider Select
    /// Bits[DB22:DB20] select the value of the RF output divider (see
    /// Figure 28).
    pub rf_divider_select: u8, // 3bits

    /// Band Select Clock Divider Value
    /// Bits[DB19:DB12] set a divider for the band select logic clock input.
    /// By default, the output of the R counter is the value used to clock
    /// the band select logic, but, if this value is too high (>125 kHz), a
    /// divider can be switched on to divide the R counter output to a
    /// smaller value (see Figure 28).
    pub band_select_clock_div: u8,

    /// VCO Power-Down
    pub vco_power_down: VcoPowerDown,

    /// Mute Till Lock Detect (MTLD)
    pub mute_till_lock_detect: MuteTillLockDetect,

    /// AUX Output Select
    pub aux_output_select: AuxOutputSelect,

    /// AUX Output Enable
    pub aux_output_enable: AuxOutputEnable,

    /// AUX Output Power
    /// Bits[DB7:DB6] set the value of the auxiliary RF output power
    /// level (see Figure 28).
    pub aux_output_power: u8, // 2bits

    /// RF Output Enable
    pub rf_output_enable: RfOutputEnable,

    /// Output Power
    /// Bits[DB4:DB3] set the value of the primary RF output power
    /// level (see Figure 28).
    pub output_power: u8, // 2bits
}

impl Reg for Reg4 {
    fn to_word(self: &Self, _pr1: Pr1Prescaler) -> u32 {
        (self.feedback_select as u32) << 23 |
        ((self.rf_divider_select & 0b111).min(6) as u32) << 20 |
        (self.band_select_clock_div.max(1) as u32) << 12 |
        (self.vco_power_down as u32) << 11 |
        (self.mute_till_lock_detect as u32) << 10 |
        (self.aux_output_select as u32) << 9 |
        (self.aux_output_enable as u32) << 8 |
        ((self.aux_output_power & 0b11) as u32) << 6 |
        (self.rf_output_enable as u32) << 5 |
        ((self.output_power & 0b11) as u32) << 3 |
        0b100
    }
}

/// Bits[DB23:DB22] set the operation of the lock detect (LD) pin
/// (see Figure 29).
#[derive(Debug,Copy,Clone)]
pub enum LockDetectPin {
    Low,
    DigitalLockDetect,
    Low1,
    High,
}

/// Register 5
#[derive(Debug,Copy,Clone)]
pub struct Reg5 {
    /// Lock Detect Pin Operation
    pub lock_detect_pin: LockDetectPin,
}
impl Reg for Reg5 {
    fn to_word(self: &Self, _pr1: Pr1Prescaler) -> u32 {
        (self.lock_detect_pin as u32) << 22 |
        0b101
    }
}
