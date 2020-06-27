#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use cortex_m_rt::{entry, exception};

use crate::hal::{
    gpio::gpioa::{PA11, PA8},
    gpio::gpiob::{PB0, PB1, PB4, PB5, PB6, PB7},
    gpio::gpiof::{PF0, PF1},
    gpio::{Output, PushPull},
    prelude::*,
    stm32,
};

use cortex_m::{
    interrupt::Mutex,
    peripheral::{syst::SystClkSource::Core, Peripherals},
};

use core::cell::RefCell;

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
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(|cs| {
            // Configure clock to 8 MHz (i.e. the default) and freeze it
            let mut rcc = p.RCC.configure().sysclk(8.mhz()).freeze(&mut p.FLASH);

            let gpioa = p.GPIOA.split(&mut rcc);
            let gpiob = p.GPIOB.split(&mut rcc);
            let gpiof = p.GPIOF.split(&mut rcc);

            // The GPIOs we use to drive the display, conveniently located at one side of the Nucleo
            // breadboard connector
            let one = gpiob.pb0.into_push_pull_output_hs(cs);
            let two = gpiob.pb7.into_push_pull_output_hs(cs);
            let three = gpiob.pb6.into_push_pull_output_hs(cs);
            let seg_a = gpiob.pb4.into_push_pull_output_hs(cs);
            let seg_b = gpiob.pb5.into_push_pull_output_hs(cs);
            let seg_c = gpioa.pa11.into_push_pull_output_hs(cs);
            let seg_d = gpioa.pa8.into_push_pull_output_hs(cs);
            let seg_e = gpiof.pf1.into_push_pull_output_hs(cs);
            let seg_f = gpiof.pf0.into_push_pull_output_hs(cs);
            let seg_g = gpiob.pb1.into_push_pull_output_hs(cs);

            let mut syst = cp.SYST;

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
fn SysTick() {
    static mut DIGIT: u8 = 0;
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
            three.set_low().ok();
            two.set_low().ok();
            one.set_low().ok();

            *DIGIT = match DIGIT {
                0 => {
                    let lsb = num % 16;
                    three.set_high().ok();
                    display.display(lsb as u8).ok();
                    1
                }
                1 => {
                    let middle = (num / 16) % 16;
                    two.set_high().ok();
                    display.display(middle as u8).ok();
                    2
                }
                2 => {
                    let msb = (num / 256) % 16;
                    one.set_high().ok();
                    display.display(msb as u8).ok();
                    0
                }
                _ => 0,
            };
        }
    });
}
