#![no_std]
#![allow(non_camel_case_types)]

pub use stm32f0xx_hal as hal;

pub use cortex_m::*;
pub use cortex_m_rt::*;
pub use crate::hal::prelude::*;
pub use crate::hal::stm32::interrupt::*;
pub use crate::hal::stm32::*;
pub use crate::hal::*;
