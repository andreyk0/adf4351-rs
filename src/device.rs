///! Device pins

use embedded_hal:: {
    digital::v2::OutputPin,
    blocking::{ delay::*, spi::*, },
};


use crate::errors::*;
use crate::register::*;

/// ADF4351 device
pub struct Adf4351<CE, LE, SPI> {
    spi: SPI,
    pin_ce: CE,
    pin_le: LE,
}


impl<CE, LE, SPI,> Adf4351<CE, LE, SPI,>
where CE: OutputPin,
      LE: OutputPin,
      SPI: Write<u8>,
{
    /// Creates the device (unconfigured, no output).
    ///
    /// `spi` - SPI device (`MOSI` => `DATA`, `CLK` => `CLK`, `CPHA` = 0)
    /// `pin_ce` - "chip enable" pin
    /// `pin_le` - "load enable" pin
    ///
    pub fn new(
        spi: SPI,
        pin_ce: CE,
        pin_le: LE,
    ) -> Self {
        Adf4351 { spi, pin_ce, pin_le, }
    }

    /// Writes all control registers out.
    /// Blocking call.
    pub fn write_register_set<Delay>(
        self: &mut Self,
        delay: &mut Delay,
        rs: &RegisterSet,
    ) -> Result<(), Error>
    where Delay: DelayUs<u16>,
    {
        for r in rs.to_words().iter().rev() {
            self.write_register(delay, *r)?;
        }
        Ok(())
    }

    /// Data is clocked into the 32-bit shift register
    /// on each rising edge of CLK. The data is clocked in MSB first.
    ///
    /// Blocking implementation.
    ///
    /// Data is transferred from the shift register to one of six latches
    /// on the rising edge of LE. The destination latch is determined by
    /// the state of the three control bits (C3, C2, and C1) in the shift
    /// register.
    #[inline(always)]
    pub fn write_register<Delay>(self: &mut Self, delay: &mut Delay, w: u32) -> Result<(), Error>
    where Delay: DelayUs<u16>,
    {
        let data = [
            ((w >> 24) & 0xFF ) as u8,
            ((w >> 16) & 0xFF ) as u8,
            ((w >>  8) & 0xFF ) as u8,
            ( w        & 0xFF ) as u8,
        ];
        self.spi.write(&data).map_err(|_| Error::Spi)?;

        delay.delay_us(5);
        self.load_enable()?;
        delay.delay_us(10);
        self.load_disable()?;
        delay.delay_us(5);

        Ok(())
    }

    /// Powers up the device, depending on the status of the power-down bits.
    #[inline(always)]
    pub fn enable(self: &mut Self) -> Result<(), Error> {
        self.pin_ce.set_high().map_err(|_| Error::Pin)
    }

    /// Powers down the device and puts the charge pump into three-state mode.
    #[inline(always)]
    pub fn disable(self: &mut Self) -> Result<(), Error> {
        self.pin_ce.set_low().map_err(|_| Error::Pin)
    }

    /// When LE goes high, the data stored in the 32-bit shift register is
    /// loaded into the register that is selected by the three control bits.
    #[inline(always)]
    pub fn load_enable(self: &mut Self) -> Result<(), Error> {
        self.pin_le.set_high().map_err(|_| Error::Pin)
    }

    /// Disable register load from shift register
    #[inline(always)]
    fn load_disable(self: &mut Self) -> Result<(), Error> {
        self.pin_le.set_low().map_err(|_| Error::Pin)
    }
}
