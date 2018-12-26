#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use cortex_m_rt::entry;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::hal::spi::{Mode, Phase, Polarity};
use crate::hal::stm32;

// the eink library
use epd_waveshare::{
    epd1in54::{Buffer1in54, EPD1in54},
    graphics::Display,
    prelude::*,
};

// Graphics
use embedded_graphics::coord::Coord;
use embedded_graphics::fonts::Font12x16;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Circle;
use embedded_graphics::Drawing;

pub const MODE: Mode = Mode {
    polarity: Polarity::IdleHigh,
    phase: Phase::CaptureOnSecondTransition,
};

#[entry]
fn main() -> ! {
    if let (Some(p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        let rcc = p.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(48.mhz()).freeze();
        let gpiob = p.GPIOB.split();
        let mut delay = Delay::new(cp.SYST, clocks);

        // Configure pins for SPI
        let sck = gpiob.pb3.into_alternate_af0();
        let miso = gpiob.pb4.into_alternate_af0();
        let mosi = gpiob.pb5.into_alternate_af0();

        let dc = gpiob.pb1.into_push_pull_output();
        let busy = gpiob.pb6.into_floating_input();
        let rst = gpiob.pb7.into_push_pull_output();
        let cs = gpiob.pb0.into_push_pull_output();

        // Configure SPI with 1MHz rate
        let mut spi = Spi::spi1(p.SPI1, (sck, miso, mosi), MODE, 8_000_000.hz(), clocks);

        let mut epd = EPD1in54::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();

        // Setup the graphics
        let mut buffer = Buffer1in54::default();
        let mut display = Display::new(epd.width(), epd.height(), &mut buffer.buffer);

        display.draw(
            Font12x16::render_str("Hello Rust!")
                .with_stroke(Some(Color::Black))
                .with_fill(Some(Color::White))
                .translate(Coord::new(5, 50))
                .into_iter(),
        );

        display.draw(
            Circle::new(Coord::new(80, 80), 80)
                .with_stroke(Some(Color::Black))
                .into_iter(),
        );

        // Transfer the frame data to the epd
        let _ = epd.update_frame(&mut spi, &display.buffer());

        // Display the frame on the epd
        let _ = epd.display_frame(&mut spi);
    }

    loop {
        continue;
    }
}
