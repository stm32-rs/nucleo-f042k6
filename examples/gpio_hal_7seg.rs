#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate panic_abort;

extern crate stm32f042_hal as hal;

use cortex_m_rt::entry;

use hal::delay::Delay;
use hal::prelude::*;
use hal::stm32f042;

use cortex_m::peripheral::Peripherals;

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32f042::Peripherals::take(), Peripherals::take()) {
        let gpioa = p.GPIOA.split();
        let gpiob = p.GPIOB.split();
        let gpiof = p.GPIOF.split();

        /* (Re-)configure PB3 as output */
        let mut one = gpiob.pb0.into_push_pull_output();
        let mut two = gpiob.pb7.into_push_pull_output();
        let mut three = gpiob.pb6.into_push_pull_output();
        let mut a = gpiob.pb4.into_push_pull_output();
        let mut b = gpiob.pb5.into_push_pull_output();
        let mut c = gpioa.pa11.into_push_pull_output();
        let mut d = gpioa.pa8.into_push_pull_output();
        let mut e = gpiof.pf1.into_push_pull_output();
        let mut f = gpiof.pf0.into_push_pull_output();
        let mut g = gpiob.pb1.into_push_pull_output();

        /* Constrain clocking registers */
        let mut rcc = p.RCC.constrain();

        /* Configure clock to 8 MHz (i.e. the default) and freeze it */
        let clocks = rcc.cfgr.sysclk(8.mhz()).freeze();

        /* Get delay provider */
        let mut delay = Delay::new(cp.SYST, clocks);

        let sevenseg = SevenSeg::new(a, b, c, d, e, f, g);
        let mut multiseg = MultiSevenSeg::new(sevenseg, one, two, three);

        loop {
            multiseg.display(456);
        }
    }

    loop {}
}

use hal::hal::digital::OutputPin;

struct SevenSeg<A, B, C, D, E, F, G> {
    a: A,
    b: B,
    c: C,
    d: D,
    e: E,
    f: F,
    g: G,
}

impl<A, B, C, D, E, F, G> SevenSeg<A, B, C, D, E, F, G>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    E: OutputPin,
    F: OutputPin,
    G: OutputPin,
{
    pub fn new(a: A, b: B, c: C, d: D, e: E, f: F, g: G) -> Self {
        Self {
            a,
            b,
            c,
            d,
            e,
            f,
            g,
        }
    }

    pub fn release(self) -> (A, B, C, D, E, F, G) {
        (self.a, self.b, self.c, self.d, self.e, self.f, self.g)
    }

    pub fn clear(&mut self) {
        self.a.set_low();
        self.b.set_low();
        self.c.set_low();
        self.d.set_low();
        self.e.set_low();
        self.f.set_low();
        self.g.set_low();
    }

    pub fn display(&mut self, num: u8) {
        match num {
            0 => {
                self.a.set_high();
                self.b.set_high();
                self.c.set_high();
                self.d.set_high();
                self.e.set_high();
                self.f.set_high();
                self.g.set_low();
            }
            1 => {
                self.a.set_low();
                self.b.set_low();
                self.c.set_low();
                self.d.set_low();
                self.e.set_high();
                self.f.set_high();
                self.g.set_low();
            }
            2 => {
                self.a.set_high();
                self.b.set_high();
                self.c.set_low();
                self.d.set_high();
                self.e.set_high();
                self.f.set_low();
                self.g.set_high();
            }
            3 => {
                self.a.set_high();
                self.b.set_low();
                self.c.set_low();
                self.d.set_high();
                self.e.set_high();
                self.f.set_high();
                self.g.set_high();
            }
            4 => {
                self.a.set_low();
                self.b.set_low();
                self.c.set_high();
                self.d.set_low();
                self.e.set_high();
                self.f.set_high();
                self.g.set_high();
            }
            5 => {
                self.a.set_high();
                self.b.set_low();
                self.c.set_high();
                self.d.set_high();
                self.e.set_low();
                self.f.set_high();
                self.g.set_high();
            }
            6 => {
                self.a.set_high();
                self.b.set_high();
                self.c.set_high();
                self.d.set_high();
                self.e.set_low();
                self.f.set_high();
                self.g.set_high();
            }
            7 => {
                self.a.set_low();
                self.b.set_low();
                self.c.set_low();
                self.d.set_high();
                self.e.set_high();
                self.f.set_high();
                self.g.set_low();
            }
            8 => {
                self.a.set_high();
                self.b.set_high();
                self.c.set_high();
                self.d.set_high();
                self.e.set_high();
                self.f.set_high();
                self.g.set_high();
            }
            9 => {
                self.a.set_high();
                self.b.set_low();
                self.c.set_high();
                self.d.set_high();
                self.e.set_high();
                self.f.set_high();
                self.g.set_high();
            }
            _ => {
                self.a.set_low();
                self.b.set_low();
                self.c.set_low();
                self.d.set_low();
                self.e.set_low();
                self.f.set_low();
                self.g.set_low();
            }
        }
    }
}

struct MultiSevenSeg<A, B, C, D, E, F, G, ONE, TWO, THREE> {
    sevenseg: SevenSeg<A, B, C, D, E, F, G>,
    one: ONE,
    two: TWO,
    three: THREE,
}

impl<A, B, C, D, E, F, G, ONE, TWO, THREE> MultiSevenSeg<A, B, C, D, E, F, G, ONE, TWO, THREE>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    E: OutputPin,
    F: OutputPin,
    G: OutputPin,
    ONE: OutputPin,
    TWO: OutputPin,
    THREE: OutputPin,
{
    pub fn new(sevenseg: SevenSeg<A, B, C, D, E, F, G>, one: ONE, two: TWO, three: THREE) -> Self {
        Self {
            sevenseg,
            one,
            two,
            three,
        }
    }

    pub fn release(self) -> (SevenSeg<A, B, C, D, E, F, G>, ONE, TWO, THREE) {
        (self.sevenseg, self.one, self.two, self.three)
    }

    pub fn display(&mut self, num: u16) {
        let digit3 = num % 10;
        let digit2 = (num / 10) % 10;
        let digit1 = (num / 100) % 10;

        self.three.set_high();
        for _ in 0..=10 {
            self.sevenseg.display(digit3 as u8);
        }
        self.three.set_low();

        self.two.set_high();
        for _ in 0..=10 {
            self.sevenseg.display(digit2 as u8);
        }
        self.two.set_low();

        self.one.set_high();
        for _ in 0..=10 {
            self.sevenseg.display(digit1 as u8);
        }
        self.one.set_low();
    }
}
