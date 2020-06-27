#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;

use crate::hal::{prelude::*, stm32};

use cortex_m::{interrupt::Mutex, peripheral::syst::SystClkSource::Core, peripheral::Peripherals};
use cortex_m_rt::{entry, exception};

use core::{cell::RefCell, fmt::Write, ptr};

struct Shared {
    adc: hal::adc::Adc,
    temp: hal::adc::VTemp,
    reference: hal::adc::VRef,
    tx: hal::serial::Tx<stm32::USART2>,
}

static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::new(RefCell::new(None));

fn calculate_temperature(reading: u16) -> i16 {
    const VDD_CALIB: i32 = 330;
    const VDD_APPLI: i32 = 300;

    let cal30 = i32::from(unsafe { ptr::read(0x1FFF_F7B8 as *const u16) });
    let cal110 = i32::from(unsafe { ptr::read(0x1FFF_F7C2 as *const u16) });

    let mut temperature: i32 = ((i32::from(reading) * VDD_APPLI) / VDD_CALIB) - cal30;
    temperature *= 110 - 30;
    temperature /= cal110 - cal30;
    temperature += 30;
    temperature as i16
}

fn calculate_vdda(reading: u16) -> u16 {
    let vrefint = u32::from(unsafe { ptr::read(0x1FFF_F7BA as *const u16) });

    // The RM says 0.3 but that's way off, 0.33 is a lot more accurate but results in a bit too
    // high reading (probably due to somewhat clipped range), 0.325 is almost spot of dor me
    (3250 * vrefint / u32::from(reading)) as u16
}

#[entry]
fn main() -> ! {
    if let (Some(mut p), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        cortex_m::interrupt::free(|cs| {
            let mut rcc = p.RCC.configure().freeze(&mut p.FLASH);
            let gpioa = p.GPIOA.split(&mut rcc);
            let mut syst = cp.SYST;

            // Set source for SysTick counter, here full operating frequency (== 8MHz)
            syst.set_clock_source(Core);

            // Set reload value, i.e. timer delay 8 MHz/counts
            syst.set_reload(8_000_000 - 1);

            // Start SysTick counter
            syst.enable_counter();

            // Start SysTick interrupt generation
            syst.enable_interrupt();

            // USART2 at PA2 (TX) and PA15(RX) is connectet to ST-Link
            let tx = gpioa.pa2.into_alternate_af1(cs);
            let rx = gpioa.pa15.into_alternate_af1(cs);

            // Initialiase UART
            let (mut tx, _) =
                hal::serial::Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), &mut rcc).split();

            // Initialise ADC
            let mut adc = hal::adc::Adc::new(p.ADC, &mut rcc);

            // Initialise core temperature sensor
            let mut temp = hal::adc::VTemp::new();

            // Initialise voltage reference sensor
            let mut reference = hal::adc::VRef::new();

            // And enable readings
            temp.enable(&mut adc);
            reference.enable(&mut adc);

            // Output a friendly greeting
            tx.write_str("\n\rThis ADC example will read various values using the ADC and print them out to the serial terminal\r\n").ok();

            // Move all components under Mutex supervision
            *SHARED.borrow(cs).borrow_mut() = Some(Shared {
                adc,
                temp,
                reference,
                tx,
            });
        });
    }

    loop {
        continue;
    }
}

#[exception]
fn SysTick() {
    use core::ops::DerefMut;

    // Enter critical section
    cortex_m::interrupt::free(|cs| {
        // Get access to the Mutex protected shared data
        if let Some(ref mut shared) = SHARED.borrow(cs).borrow_mut().deref_mut() {
            // Read raw temperature data from internal sensor using ADC
            let t: Result<u16, _> = shared.adc.read(&mut shared.temp);
            if let Ok(t) = t {
                // Calculate accurate value and print it
                writeln!(shared.tx, "Temperature {}\r", calculate_temperature(t)).ok();
            } else {
                shared.tx.write_str("Error reading temperature").ok();
            }

            // Read raw volatage reference data from internal sensor using ADC
            let t: Result<u16, _> = shared.adc.read(&mut shared.reference);
            if let Ok(t) = t {
                // Calculate accurate value and print it
                writeln!(shared.tx, "Vdda {}mV\r", calculate_vdda(t)).ok();
            } else {
                shared.tx.write_str("Error reading Vdda").ok();
            }
        }
    });
}
