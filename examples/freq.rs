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

        // Charge pump
        .set(ChargePumpCurrent(0b111))

        // Phase
        .set(PhaseDetectorPolarity::Positive)

        // FRAC-N modulus
        .set(Mod(4000))

        // keep xtal input, clean up duty cycle
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
