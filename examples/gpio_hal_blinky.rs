#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_abort;

extern crate stm32f042_hal as hal;

use cortex_m_rt::entry;

use hal::delay::Delay;
use hal::prelude::*;
use hal::stm32;

use cortex_m::peripheral::Peripherals;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let gpiob = p.GPIOB.split();

        /* (Re-)configure PB3 as output */
        let mut led = gpiob.pb3.into_push_pull_output();

        /* Constrain clocking registers */
        let mut rcc = p.RCC.constrain();

        /* Configure clock to 8 MHz (i.e. the default) and freeze it */
        let clocks = rcc.cfgr.sysclk(8.mhz()).freeze();

        /* Get delay provider */
        let mut delay = Delay::new(cp.SYST, clocks);

        loop {
            led.set_high();
            delay.delay_ms(1_000_u16);

            led.set_low();
            delay.delay_ms(1_000_u16);
        }
    }

    loop {}
}
