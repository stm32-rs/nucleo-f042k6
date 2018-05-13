#![feature(used)]
#![no_main]
#![no_std]

#[macro_use(entry, exception)]
extern crate cortex_m_rt;

use cortex_m_rt::ExceptionFrame;

extern crate cortex_m;
extern crate panic_abort;
extern crate stm32f042_hal as hal;

extern crate embedded_hal;
//use embedded_hal::blocking::i2c::Write;

extern crate ssd1306;
use ssd1306::mode::TerminalMode;
use ssd1306::Builder;

use hal::i2c::*;
use hal::prelude::*;
use hal::stm32f042;

use core::fmt::Write;

exception!(*, default_handler);

fn default_handler(_irqn: i16) {}

exception!(HardFault, hard_fault);

fn hard_fault(_ef: &ExceptionFrame) -> ! {
    loop {}
}

entry!(main);

fn main() -> ! {
    if let Some(p) = stm32f042::Peripherals::take() {
        let gpiof = p.GPIOF.split();
        let mut rcc = p.RCC.constrain();
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

        /* Setup I2C1 */
        let mut i2c = I2c::i2c1(p.I2C1, (scl, sda), 400.khz());

        use ssd1306::displayrotation::DisplayRotation;
        let mut disp: TerminalMode<_> = Builder::new().with_i2c_addr(0x3c).connect_i2c(i2c).into();

        let _ = disp.set_rotation(DisplayRotation::Rotate180);
        disp.init().unwrap();
        let _ = disp.clear();
        /* Endless loop */
        loop {
            for c in 97..123 {
                let _ = disp.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
            }
            for c in 65..91 {
                let _ = disp.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
            }
        }
    }

    loop {}
}
