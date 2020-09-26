#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate panic_halt; // panic handler

use cortex_m;
use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use cortex_m_semihosting::hprintln;

use crate::hal::{
    prelude::*,
    stm32,
    spi::Spi,
};

use embedded_hal::spi::MODE_0;

use adf4351::{ device::*, register::*, config::* };

/// Example board config / test frequency generator.
/// Example boards:
/// [STM32F407VET6](https://www.amazon.com/HiLetgo-STM32F407VET6-Cortex-M4-Development-NRF2410/dp/B071KBZR58/ref=sxts_sxwds-bia-wc-drs-ajax1_0?cv_ct_cx=stm32f4&dchild=1&keywords=stm32f4&pd_rd_i=B071KBZR58&pd_rd_r=55e7d3f3-cdc8-4579-86db-7ca754be35b7&pd_rd_w=A8TgI&pd_rd_wg=J3B78&pf_rd_p=037ca9fd-790e-4a16-836b-14da89aed20e&pf_rd_r=1R23WCQBWJCYQN1MXC0G&psc=1&qid=1601162000&sr=1-1-25b07e09-600a-4f0d-816e-b06387f8bcf1)
/// [ADF4351](https://www.amazon.com/35M-4-4GHz-Frequency-Synthesizer-Development-Generator/dp/B078NRD8V6/ref=sr_1_3?dchild=1&keywords=adf4351&qid=1601162186&sr=8-3)
#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(8.mhz()).sysclk(168.mhz()).pclk1(42.mhz()).pclk2(84.mhz()).freeze();

    let gpioa = dp.GPIOA.split();
    let mut led1 = gpioa.pa6.into_push_pull_output();

    let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    let gpiob = dp.GPIOB.split();
    let pin_ce = gpiob.pb10.into_push_pull_output();
    let pin_le = gpiob.pb11.into_push_pull_output();

    let sck = gpiob.pb13.into_alternate_af5();
    let mosi = gpiob.pb15.into_alternate_af5();

    let spi = Spi::spi2(
        dp.SPI2,
        (sck, hal::spi::NoMiso , mosi),
        MODE_0,
        stm32f4xx_hal::time::KiloHertz(100).into(),
        clocks,
    );

    let mut sg = Adf4351::new(spi, pin_ce, pin_le);
    sg.enable().unwrap();
    let xtal = 25_000_000;

    let f = 63_000_000;
    let rs = RegisterSet::default()
        // Double buffer register writes
        .set(DoubleBuffer::Enabled)

        // FRAC-N modulus
        .set(Mod(4000))

        // Board config
        .set(ChargePumpCurrent(0b111))
        .set(PhaseDetectorPolarity::Positive)

        // Keep xtal input, clean up duty cycle
        .set(R(1))
        .set(RefDoubler::Enabled)
        .set(Rdiv2::Enabled)

        // Pin config
        .set(LockDetectPin::DigitalLockDetect)

        // RF out
        .set(BandSelectClockDiv(200))
        .set(AuxOutputEnable::Enabled)
        .set(AuxOutputPower(2))
        .set(RfOutputEnable::Enabled)
        .set(OutputPower(2))
        ;

    let rs = FracN::init(rs); // init FracN mode, one time settings
    let fracn = FracN(Fpfd::new(xtal, &rs).unwrap()); // init with cuffent PFD config
    let rs = fracn.set_f_out(f, rs).unwrap(); // set output frequency

    sg.write_register_set(&mut delay, &rs).unwrap();

    let rs_words = rs.to_words();
    for (i,w) in rs_words.iter().enumerate() {
        hprintln!("RS[{}] {:#010x} {:#034b}", i, w, w).unwrap();
    };

    let int : Int = rs.get();
    let frac : Frac = rs.get();
    let modulus : Mod = rs.get();
    let rfdiv : RfDividerSelect = rs.get();
    hprintln!("{:?} {:?} {:?} {:?}", int, frac, modulus, rfdiv).unwrap();
    hprintln!("{:?} => f {:?} <-> f_out {:?}", fracn.0, f, FracN::f_out_hz(xtal, &rs)).unwrap();

    loop {
        led1.set_high().unwrap();
        delay.delay_ms(1000_u32);
        led1.set_low().unwrap();
        delay.delay_ms(1000_u32);
    }
}
