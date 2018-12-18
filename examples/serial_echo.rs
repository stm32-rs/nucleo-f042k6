#![no_main]
#![no_std]

use panic_halt;

use stm32f0xx_hal as hal;

use nb::block;

use crate::hal::prelude::*;
use crate::hal::stm32;

use crate::hal::serial::Serial;

use cortex_m_rt::entry;

#[entry]
fn main() -> ! {
    if let Some(p) = stm32::Peripherals::take() {
        let gpioa = p.GPIOA.split();
        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();

        // USART2 at PA2 (TX) and PA15(RX) is connectet to ST-Link
        let tx = gpioa.pa2.into_alternate_af1();
        let rx = gpioa.pa15.into_alternate_af1();

        let serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), clocks);

        let (mut tx, mut rx) = serial.split();

        loop {
            let received = block!(rx.read()).unwrap();
            block!(tx.write(received)).ok();
        }
    }

    loop {
        continue;
    }
}
