#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use cortex_m_rt::entry;

use crate::hal::{
    gpio::{gpiof::PF0, gpiof::PF1, Alternate, AF1},
    i2c::*,
    prelude::*,
    serial::Serial,
    stm32::{self, interrupt, Interrupt::USART2},
};

use cortex_m::interrupt::Mutex;

use core::{cell::RefCell, fmt::Write, ops::DerefMut};

// Make some peripherals globally available
struct Shared {
    i2c: hal::i2c::I2c<stm32::I2C1, PF1<Alternate<AF1>>, PF0<Alternate<AF1>>>,
    rx: hal::serial::Rx<stm32::USART2>,
    tx: hal::serial::Tx<stm32::USART2>,
}

static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        cortex_m::interrupt::free(|cs| {
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);
            let gpioa = p.GPIOA.split(&mut rcc);
            let gpiof = p.GPIOF.split(&mut rcc);
            let mut nvic = cp.NVIC;

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
            let i2c = I2c::i2c1(p.I2C1, (scl, sda), 100.khz(), &mut rcc);

            // USART2 at PA2 (TX) and PA15(RX) is connectet to ST-Link
            let tx = gpioa.pa2.into_alternate_af1(cs);
            let rx = gpioa.pa15.into_alternate_af1(cs);

            // Set up our serial port for output
            let mut serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), &mut rcc);

            // Enable USART2 interrupt on received input
            serial.listen(hal::serial::Event::Rxne);
            let (mut tx, rx) = serial.split();

            // Enable USART2 interrupt and clear any pending interrupts
            nvic.enable(USART2);
            cortex_m::peripheral::NVIC::unpend(USART2);

            // Print a welcome message
            tx.write_str("\r\nWelcome to the I2C scanner. Enter any character to start scan.\r\n")
                .ok();

            // Move all components under Mutex supervision
            *SHARED.borrow(cs).borrow_mut() = Some(Shared { i2c, rx, tx });
        });
    }

    loop {
        continue;
    }
}

// The IRQ handler triggered by a received character in USART buffer, this will conduct our I2C
// scan when we receive anything
#[interrupt]
fn USART2() {
    cortex_m::interrupt::free(|cs| {
        // Obtain all Mutex protected resources
        if let Some(ref mut shared) = SHARED.borrow(cs).borrow_mut().deref_mut() {
            let tx = &mut shared.tx;
            let rx = &mut shared.rx;
            let i2c = &mut shared.i2c;

            /* Read the character that triggered the interrupt from the USART */
            while rx.read().is_ok() {}

            /* Output address schema for tried addresses */
            let _ = tx.write_str("\r\n");
            let _ = tx.write_str(
                "0               1               2               3               4               5               6               7\r\n",
                );
            let _ = tx.write_str(
                "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF\r\n",
                );

            // Execute scanning once for each valid I2C address
            for addr in 0..=0x7f {
                let res = i2c.write(addr, &[0]);

                // If we received a NACK there's no device on the attempted address
                let _ = tx.write_str(match res {
                    Err(Error::NACK) => ".",
                    _ => "Y",
                });
            }

            let _ = tx.write_str(
                "\r\n\r\nScan done.\r\n'Y' means a device was found on the I2C address above.\r\n'.' means no device found on that address.\r\nPlease enter any character to start a new scan.\r\n",
                );
        }

        // Clear interrupt flag
        cortex_m::peripheral::NVIC::unpend(USART2);
    });
}
