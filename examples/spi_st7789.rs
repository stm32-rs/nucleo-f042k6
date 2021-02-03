#![no_main]
#![no_std]

use panic_halt as _;

use stm32f0xx_hal as hal;

use crate::hal::{
    delay::Delay,
    prelude::*,
    serial::Serial,
    spi::Spi,
    spi::{Mode, Phase, Polarity},
    stm32 as pac,
    stm32::{interrupt, Interrupt, TIM2},
    timers::{Event, Timer},
};

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::style::*;

use core::cell::RefCell;
use core::fmt::Write as _;
use core::ops::DerefMut;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;

// Make timer interrupt registers globally available
static GINT: Mutex<RefCell<Option<Timer<TIM2>>>> = Mutex::new(RefCell::new(None));

#[derive(Copy, Clone)]
struct Time {
    seconds: u32,
    millis: u16,
}

static TIME: Mutex<RefCell<Time>> = Mutex::new(RefCell::new(Time {
    seconds: 0,
    millis: 0,
}));

// Define an interupt handler, i.e. function to call when interrupt occurs. Here if our external
// interrupt trips when the timer timed out
#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        // Move LED pin here, leaving a None in its place
        GINT.borrow(cs)
            .borrow_mut()
            .deref_mut()
            .as_mut()
            .unwrap()
            .wait()
            .ok();
        let mut time = TIME.borrow(cs).borrow_mut();
        time.millis += 1;
        if time.millis == 1000 {
            time.millis = 0;
            time.seconds += 1;
        }
    });
}

#[entry]
fn main() -> ! {
    const MODE: Mode = Mode {
        polarity: Polarity::IdleHigh,
        phase: Phase::CaptureOnSecondTransition,
    };

    if let (Some(p), Some(cp)) = (pac::Peripherals::take(), cortex_m::Peripherals::take()) {
        let (mut serial, mut display) = cortex_m::interrupt::free(move |cs| {
            let mut flash = p.FLASH;
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);

            // Use USART2 with PA2 and PA3 as serial port
            let gpioa = p.GPIOA.split(&mut rcc);
            let tx = gpioa.pa2.into_alternate_af1(cs);
            let rx = gpioa.pa15.into_alternate_af1(cs);

            let gpiob = p.GPIOB.split(&mut rcc);

            // Initialise delay provider
            let mut delay = Delay::new(cp.SYST, &rcc);

            // Configure pins for SPI
            let sck = gpiob.pb13.into_alternate_af0(cs);
            let miso = gpiob.pb14.into_alternate_af0(cs);
            let mosi = gpiob.pb15.into_alternate_af0(cs);
            let dc = gpiob.pb1.into_push_pull_output(cs);
            let rst = gpiob.pb2.into_push_pull_output(cs);

            // Set up a timer expiring every millisecond
            let mut timer = Timer::tim2(p.TIM2, 1000.hz(), &mut rcc);

            // Generate an interrupt when the timer expires
            timer.listen(Event::TimeOut);

            // Move the timer into our global storage
            *GINT.borrow(cs).borrow_mut() = Some(timer);

            // Set up our serial port
            let serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), &mut rcc);

            // Configure SPI with 24MHz rate
            let spi = Spi::spi2(p.SPI2, (sck, miso, mosi), MODE, 24_000_000.hz(), &mut rcc);
            //let mut spi = Spi::spi2(p.SPI2, (sck, miso, mosi), MODE, 24_000_000.hz(), &mut rcc).into_8bit_width();

            // create driver
            // let interface = display_interface_spi::SPIInterfaceNoCS::new(spi, dc);
            //let mut display = ST7789::new(interface, rst, 240, 240);

            let interface = display_interface_spi::SPIInterfaceNoCS::new(spi, dc);
            let mut display = ili9341::Ili9341::new(interface, rst, &mut delay).unwrap();

            // initialize
            //display.init(&mut delay).unwrap();
            // set default orientation
            //display.set_orientation(Orientation::Landscape).unwrap();

            // Enable TIM7 IRQ, set prio 1 and clear any pending IRQs
            let mut nvic = cp.NVIC;
            unsafe {
                nvic.set_priority(Interrupt::TIM7, 1);
                cortex_m::peripheral::NVIC::unmask(Interrupt::TIM7);
            }
            cortex_m::peripheral::NVIC::unpend(Interrupt::TIM7);

            (serial, display)
        });

        let circle1 = Circle::new(Point::new(128, 64), 64)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::RED));
        let circle2 = Circle::new(Point::new(64, 64), 64)
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1));

        let blue_with_red_outline = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLUE)
            .stroke_color(Rgb565::RED)
            .stroke_width(1) // > 1 is not currently suppored in embedded-graphics on triangles
            .build();
        let triangle = Triangle::new(
            Point::new(40, 120),
            Point::new(40, 220),
            Point::new(140, 120),
        )
        .into_styled(blue_with_red_outline);

        let line = Line::new(Point::new(180, 160), Point::new(239, 239))
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 10));

        loop {
            display.clear(Rgb565::BLACK).unwrap();

            cortex_m::interrupt::free(|cs| {
                let mut time = TIME.borrow(cs).borrow_mut();

                // Print the current time
                writeln!(serial, "blank: {}.{:03}s\r", time.seconds, time.millis).ok();

                // Reset the time
                time.millis = 0;
                time.seconds = 0;
            });

            // draw two circles on blue background
            circle1.draw(&mut display).unwrap();
            circle2.draw(&mut display).unwrap();
            triangle.draw(&mut display).unwrap();
            line.draw(&mut display).unwrap();

            cortex_m::interrupt::free(|cs| {
                let mut time = TIME.borrow(cs).borrow_mut();

                // Print the current time
                writeln!(serial, "{}.{:03}s\r", time.seconds, time.millis).ok();

                // Reset the time
                time.millis = 0;
                time.seconds = 0;
            });
        }
    }

    loop {
        continue;
    }
}
