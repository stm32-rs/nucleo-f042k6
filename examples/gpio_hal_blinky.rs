#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use cortex_m_rt::entry;

use crate::hal::{delay::Delay, prelude::*, stm32};

use cortex_m::peripheral::Peripherals;

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let (mut led, mut delay) = cortex_m::interrupt::free(|cs| {
            // Configure clock to 8 MHz (i.e. the default) and freeze it
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

            // Obtain resources from GPIO port B
            let gpiob = p.GPIOB.split(&mut rcc);

            // (Re-)configure PB3 as output
            let led = gpiob.pb3.into_push_pull_output(cs);

            // Get delay provider
            let delay = Delay::new(cp.SYST, &rcc);

            (led, delay)
        });

        loop {
            led.toggle().ok();
            delay.delay_ms(1_000_u16);
        }
    }

    loop {
        continue;
    }
}
