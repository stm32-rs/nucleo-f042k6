#![no_main]
#![no_std]

use panic_halt;

use stm32f0xx_hal as hal;

use cortex_m_rt::entry;
use ssd1306::mode::TerminalMode;
use ssd1306::Builder;

use crate::hal::i2c::*;
use crate::hal::prelude::*;
use crate::hal::stm32;

use core::fmt::Write;

#[entry]
fn main() -> ! {
    if let Some(p) = stm32::Peripherals::take() {
        let gpiof = p.GPIOF.split();
        let rcc = p.RCC.constrain();
        let _ = rcc.cfgr.freeze();

        let scl = gpiof
            .pf1
            .into_alternate_af1()
            .internal_pull_up(true)
            .set_open_drain();
        let sda = gpiof
            .pf0
            .into_alternate_af1()
            .internal_pull_up(true)
            .set_open_drain();

        // Setup I2C1
        let i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz());

        use ssd1306::displayrotation::DisplayRotation;
        let mut disp: TerminalMode<_> = Builder::new().with_i2c_addr(0x3c).connect_i2c(i2c).into();

        let _ = disp.set_rotation(DisplayRotation::Rotate180);
        disp.init().unwrap();
        let _ = disp.clear();
        let _ = write!(disp, "Hello world!");
    }

    loop {
        continue;
    }
}
