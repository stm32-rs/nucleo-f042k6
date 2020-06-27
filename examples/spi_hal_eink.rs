#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use cortex_m_rt::entry;

use crate::hal::{
    delay::Delay,
    prelude::*,
    spi::Spi,
    spi::{Mode, Phase, Polarity},
    stm32,
};

// the eink library
use epd_waveshare::{
    color::Black,
    epd1in54::{Display1in54, EPD1in54},
    graphics::Display,
    prelude::*,
};

// Graphics
use embedded_graphics::fonts::{Font12x16, Text};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::style::{PrimitiveStyle, TextStyleBuilder};

pub const MODE: Mode = Mode {
    polarity: Polarity::IdleHigh,
    phase: Phase::CaptureOnSecondTransition,
};

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), cortex_m::Peripherals::take()) {
        cortex_m::interrupt::free(|cs| {
            let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut p.FLASH);
            let gpiob = p.GPIOB.split(&mut rcc);
            let mut delay = Delay::new(cp.SYST, &rcc);

            // Configure pins for SPI
            let sck = gpiob.pb3.into_alternate_af0(cs);
            let miso = gpiob.pb4.into_alternate_af0(cs);
            let mosi = gpiob.pb5.into_alternate_af0(cs);

            let dc = gpiob.pb1.into_push_pull_output(cs);
            let busy = gpiob.pb6.into_floating_input(cs);
            let rst = gpiob.pb7.into_push_pull_output(cs);
            let cs = gpiob.pb0.into_push_pull_output(cs);

            // Configure SPI with 1MHz rate
            let mut spi = Spi::spi1(p.SPI1, (sck, miso, mosi), MODE, 8_000_000.hz(), &mut rcc);

            let mut epd = EPD1in54::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();

            // Setup the graphics
            let mut display = Display1in54::default();

            let text_style = TextStyleBuilder::new(Font12x16)
                .text_color(BinaryColor::On)
                .build();

            Text::new("Hello Rust!", Point::new(5, 50))
                .into_styled(text_style)
                .draw(&mut display)
                .ok();

            Circle::new(Point::new(80, 80), 80)
                .into_styled(PrimitiveStyle::with_stroke(Black, 1))
                .draw(&mut display)
                .ok();

            // Transfer the frame data to the epd
            let _ = epd.update_frame(&mut spi, &display.buffer());

            // Display the frame on the epd
            let _ = epd.display_frame(&mut spi);
        });
    }

    loop {
        continue;
    }
}
