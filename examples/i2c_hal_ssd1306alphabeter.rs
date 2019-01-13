#![no_main]
#![no_std]

use stm32f0xx_hal as hal;

use cortex_m_rt::entry;
use ssd1306::mode::TerminalMode;
use ssd1306::Builder;

use crate::hal::{gpio::*, i2c::*, prelude::*, serial::*, stm32};
use cortex_m::interrupt::Mutex;

use core::{cell::RefCell, fmt::Write, ops::DerefMut};

// Make the write part of our serial port globally available
static PANIC_SERIAL: Mutex<
    RefCell<
        Option<
            hal::serial::Serial<
                stm32::USART2,
                gpioa::PA2<Alternate<AF1>>,
                gpioa::PA15<Alternate<AF1>>,
            >,
        >,
    >,
> = Mutex::new(RefCell::new(None));

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::free(|cs| {
        // Obtain mutex protected write part of serial port
        if let Some(ref mut tx) = *PANIC_SERIAL.borrow(cs).borrow_mut().deref_mut() {
            writeln!(tx, "\r\n{}", info).ok();
        }

        loop {
            continue;
        }
    })
}

#[entry]
fn main() -> ! {
    if let Some(mut p) = stm32::Peripherals::take() {
        let mut disp = cortex_m::interrupt::free(|cs| {
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
            let gpioa = p.GPIOA.split(&mut rcc);
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

            // USART2 at PA2 (TX) and PA15(RX) is connectet to ST-Link
            let tx = gpioa.pa2.into_alternate_af1(cs);
            let rx = gpioa.pa15.into_alternate_af1(cs);

            let serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), &mut rcc);

            // Transfer write part of serial port into Mutex
            *PANIC_SERIAL.borrow(cs).borrow_mut() = Some(serial);

            use ssd1306::displayrotation::DisplayRotation;
            let mut disp: TerminalMode<_> =
                Builder::new().with_i2c_addr(0x3c).connect_i2c(i2c).into();

            let _ = disp.set_rotation(DisplayRotation::Rotate180);
            disp.init().unwrap();
            let _ = disp.clear();

            disp
        });

        // Endless loop
        loop {
            for c in 97..123 {
                let _ = disp.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
            }
            for c in 65..91 {
                let _ = disp.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
            }
        }
    }

    loop {
        continue;
    }
}
