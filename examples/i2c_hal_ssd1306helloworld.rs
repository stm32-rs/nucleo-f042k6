#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use cortex_m_rt::entry;
use ssd1306::{mode::TerminalMode, Builder};

use crate::hal::{i2c::*, prelude::*, stm32};

use core::fmt::Write;

#[entry]
fn main() -> ! {
    if let Some(mut p) = stm32::Peripherals::take() {
        cortex_m::interrupt::free(|cs| {
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);
            let gpiof = p.GPIOF.split(&mut rcc);

            let scl = gpiof
                .pf1
                .into_alternate_af1(cs)
                .internal_pull_up(cs, true)
                .set_open_drain(cs);
            let sda = gpiof
                .pf0
                .into_alternate_af1(cs)
                .internal_pull_up(cs, true)
                .set_open_drain(cs);

            // Setup I2C1
            let i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz(), &mut rcc);

            use ssd1306::displayrotation::DisplayRotation;
            let mut disp: TerminalMode<_> =
                Builder::new().with_i2c_addr(0x3c).connect_i2c(i2c).into();

            disp.set_rotation(DisplayRotation::Rotate180).ok();
            disp.init().unwrap();
            disp.clear().ok();
            write!(disp, "Hello world!").ok();
        });
    }

    loop {
        continue;
    }
}
