#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::prelude::*;
use crate::hal::stm32::{self, interrupt, Interrupt::USART2};

use crate::hal::serial::Serial;

use core::cell::RefCell;
use core::fmt::Write;
use core::ops::DerefMut;

use cortex_m::interrupt::Mutex;

use cortex_m_rt::entry;

// Make some peripherals globally available
struct Shared {
    rx: hal::serial::Rx<stm32::USART2>,
    tx: hal::serial::Tx<stm32::USART2>,
}

static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        let gpioa = p.GPIOA.split();
        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
        let mut nvic = cp.NVIC;

        // USART2 at PA2 (TX) and PA15(RX) is connectet to ST-Link
        let tx = gpioa.pa2.into_alternate_af1();
        let rx = gpioa.pa15.into_alternate_af1();

        // Set up serial port
        let mut serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), clocks);

        // Enable interrupt generation for received data
        serial.listen(hal::serial::Event::Rxne);
        let (mut tx, rx) = serial.split();

        // Output a nice message
        tx.write_str("\r\nTry typing some characters and watch them being echoed.\r\n")
            .ok();

        // Move all components under Mutex supervision
        cortex_m::interrupt::free(move |cs| {
            *SHARED.borrow(cs).borrow_mut() = Some(Shared { rx, tx });
        });

        // Enable USART IRQ and clear any pending IRQs
        nvic.enable(USART2);
        cortex_m::peripheral::NVIC::unpend(USART2);
    }

    loop {
        // Power down a bit while waiting for interrupts
        cortex_m::asm::wfi();
    }
}

// The IRQ handler triggered by a received character in USART buffer
#[interrupt]
fn USART2() {
    cortex_m::interrupt::free(|cs| {
        // Obtain all Mutex protected resources
        if let Some(ref mut shared) = SHARED.borrow(cs).borrow_mut().deref_mut() {
            let tx = &mut shared.tx;
            let rx = &mut shared.rx;

            // Read received character
            let received = rx.read().unwrap();

            // Write character back
            tx.write(received).ok();

            // Clear interrupt
            cortex_m::peripheral::NVIC::unpend(USART2);
        }
    });
}
