#![no_std]
#![cfg_attr(feature = "rt", feature(global_asm))]
#![cfg_attr(feature = "rt", feature(use_extern_macros))]
#![cfg_attr(feature = "rt", feature(used))]
#![feature(const_fn)]
#![allow(non_camel_case_types)]

pub extern crate stm32f042;
pub extern crate stm32f042_hal as hal;

extern crate bare_metal;
extern crate cortex_m;
extern crate cortex_m_rt;
extern crate vcell;

pub use cortex_m::*;
pub use cortex_m_rt::*;
pub use hal::*;
pub use stm32f042::interrupt::*;
pub use stm32f042::*;
