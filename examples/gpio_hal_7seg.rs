#![no_main]
#![no_std]

use panic_halt;

use stm32f0xx_hal as hal;

use cortex_m_rt::{entry, exception};

use crate::hal::gpio::gpioa::{PA11, PA8};
use crate::hal::gpio::gpiob::{PB0, PB1, PB4, PB5, PB6, PB7};
use crate::hal::gpio::gpiof::{PF0, PF1};
use crate::hal::gpio::{Output, PushPull};
use crate::hal::prelude::*;
use crate::hal::stm32;

use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::syst::SystClkSource::Core;
use cortex_m::peripheral::Peripherals;

use core::cell::RefCell;

extern crate sevensegment;
use sevensegment::*;

// Define the Mutex so we can share our display with the interrupt handler, blerk
static DISPLAY: Mutex<
    RefCell<
        Option<
            SevenSeg<
                PB4<Output<PushPull>>,
                PB5<Output<PushPull>>,
                PA11<Output<PushPull>>,
                PA8<Output<PushPull>>,
                PF1<Output<PushPull>>,
                PF0<Output<PushPull>>,
                PB1<Output<PushPull>>,
            >,
        >,
    >,
> = Mutex::new(RefCell::new(None));

// The pins we use for the 3 digits on this display
static ONE: Mutex<RefCell<Option<PB0<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static TWO: Mutex<RefCell<Option<PB7<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static THREE: Mutex<RefCell<Option<PB6<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));

// The number we want to display
static STATE: Mutex<RefCell<u16>> = Mutex::new(RefCell::new(0));

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let mut syst = cp.SYST;
        let gpioa = p.GPIOA.split();
        let gpiob = p.GPIOB.split();
        let gpiof = p.GPIOF.split();

        // The GPIOs we use to drive the display, conveniently located at one side of the Nucleo
        // breadboard connector
        let one = gpiob.pb0.into_push_pull_output_hs();
        let two = gpiob.pb7.into_push_pull_output_hs();
        let three = gpiob.pb6.into_push_pull_output_hs();
        let seg_a = gpiob.pb4.into_push_pull_output_hs();
        let seg_b = gpiob.pb5.into_push_pull_output_hs();
        let seg_c = gpioa.pa11.into_push_pull_output_hs();
        let seg_d = gpioa.pa8.into_push_pull_output_hs();
        let seg_e = gpiof.pf1.into_push_pull_output_hs();
        let seg_f = gpiof.pf0.into_push_pull_output_hs();
        let seg_g = gpiob.pb1.into_push_pull_output_hs();

        // Constrain clocking registers
        let rcc = p.RCC.constrain();

        // Configure clock to 8 MHz (i.e. the default) and freeze it
        let _ = rcc.cfgr.sysclk(8.mhz()).freeze();

        // Set source for SysTick counter, here full operating frequency (== 8MHz)
        syst.set_clock_source(Core);

        // Set reload value, i.e. timer delay 8 MHz/counts
        syst.set_reload(60_000 - 1);

        // Start SysTick counter
        syst.enable_counter();

        // Start SysTick interrupt generation
        syst.enable_interrupt();

        // Assign the segments of the 7 segments display to driver
        let sevenseg = SevenSeg::new(seg_a, seg_b, seg_c, seg_d, seg_e, seg_f, seg_g);

        // Move driver handle and digit enable pins into Mutexes
        cortex_m::interrupt::free(move |cs| {
            *DISPLAY.borrow(cs).borrow_mut() = Some(sevenseg);
            *ONE.borrow(cs).borrow_mut() = Some(one);
            *TWO.borrow(cs).borrow_mut() = Some(two);
            *THREE.borrow(cs).borrow_mut() = Some(three);
        });

        // Increase a counter that will be displayed
        let mut counter = 0;
        loop {
            counter += 1;

            if counter % 65536 == 0 {
                cortex_m::interrupt::free(move |cs| {
                    *STATE.borrow(cs).borrow_mut() += 1;
                });
            }
        }
    }

    loop {
        continue;
    }
}

#[exception]
fn SysTick() -> ! {
    static mut digit: u8 = 0;
    use core::ops::{Deref, DerefMut};

    // Enter critical section
    cortex_m::interrupt::free(|cs| {
        let num = *STATE.borrow(cs).borrow().deref();
        let display = &*DISPLAY.borrow(cs);
        let one = &*ONE.borrow(cs);
        let two = &*TWO.borrow(cs);
        let three = &*THREE.borrow(cs);
        if let (Some(ref mut display), Some(ref mut one), Some(ref mut two), Some(ref mut three)) = (
            display.borrow_mut().deref_mut(),
            one.borrow_mut().deref_mut(),
            two.borrow_mut().deref_mut(),
            three.borrow_mut().deref_mut(),
        ) {
            display.display(17);
            three.set_low();
            two.set_low();
            one.set_low();

            *digit = match digit {
                0 => {
                    let lsb = num % 16;
                    three.set_high();
                    display.display(lsb as u8);
                    1
                }
                1 => {
                    let middle = (num / 16) % 16;
                    two.set_high();
                    display.display(middle as u8);
                    2
                }
                2 => {
                    let msb = (num / 256) % 16;
                    one.set_high();
                    display.display(msb as u8);
                    0
                }
                _ => 0,
            };
        }
    });
}
