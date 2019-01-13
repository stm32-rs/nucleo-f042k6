#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use nb::block;

use crate::hal::{prelude::*, serial::Serial, stm32};
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(mut p) = stm32::Peripherals::take() {
        cortex_m::interrupt::free(|cs| {
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
            let gpioa = p.GPIOA.split(&mut rcc);

            // USART2 at PA2 (TX) and PA15(RX) is connectet to ST-Link
            let tx = gpioa.pa2.into_alternate_af1(cs);
            let rx = gpioa.pa15.into_alternate_af1(cs);

            let mut serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), &mut rcc);

            loop {
                let received = block!(serial.read()).unwrap();
                block!(serial.write(received)).ok();
            }
        });
    }

    loop {
        continue;
    }
}
