#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;

extern crate embedded_hal;
extern crate ssd1306;
extern crate stm32f0xx_hal as hal;

use cortex_m_rt::entry;
use ssd1306::mode::TerminalMode;
use ssd1306::Builder;

use crate::hal::i2c::*;
use crate::hal::prelude::*;
use crate::hal::stm32;
use crate::hal::serial::Serial;

use core::fmt::Write;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;

use crate::hal::stm32::USART2;
use core::ops::DerefMut;

// Make the write part of our serial port globally available
static PANIC_SERIAL: Mutex<RefCell<Option<hal::serial::Tx<USART2>>>> =
    Mutex::new(RefCell::new(None));

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::free(|cs| {
        // Obtain mutex protected write part of serial port
        if let &mut Some(ref mut tx) = PANIC_SERIAL.borrow(cs).borrow_mut().deref_mut() {
            writeln!(tx, "\r\n{}", info).ok();
        }

        loop {
            continue;
        }
    })
}


#[entry]
fn main() -> ! {
    if let Some(p) = stm32::Peripherals::take() {
        let gpioa = p.GPIOA.split();
        let gpiof = p.GPIOF.split();
        let mut rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.freeze();

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

        /* USART2 at PA2 (TX) and PA15(RX) is connectet to ST-Link */
        let tx = gpioa.pa2.into_alternate_af1();
        let rx = gpioa.pa15.into_alternate_af1();

        let serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), clocks);

        let (mut tx, mut _rx) = serial.split();

        writeln!(tx, "Hello");

        // Transfer write part of serial port into Mutex
        cortex_m::interrupt::free(|cs| {
            *PANIC_SERIAL.borrow(cs).borrow_mut() = Some(tx);
        });


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
