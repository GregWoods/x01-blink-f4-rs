//! Blinks an LED
//!
//! This assumes that a LED is connected to pc13 as is the case on the blue pill board.
//!
//! Note: Without additional hardware, PC13 should not be used to drive an LED, see page 5.1.2 of
//! the reference manual for an explanation. This is not an issue on the blue pill.

#![deny(unsafe_code)]
#![no_std]
#![no_main]

use panic_halt as _;

use nb::block;

use stm32f4xx_hal::{
    prelude::*,
    stm32,
    timer::Timer,
};
//use cortex_m_semihosting::hprintln;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;

#[entry]
fn main() -> ! {
    // Get access to the core peripherals from the cortex-m crate
    let cp = cortex_m::Peripherals::take().unwrap();
    // Get access to the device specific peripherals from the peripheral access crate
    let dp = stm32::Peripherals::take().unwrap();

    // Take ownership over the raw rcc device and convert it into the corresponding
    // HAL structs
    let rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    let clocks = rcc.cfgr.freeze();

    // Acquire the GPIOC peripheral
    let gpioc = dp.GPIOC.split();

    // Configure gpio C pin 13 as a push-pull output
    let mut led = gpioc.pc13.into_push_pull_output();
    // Configure the syst timer to trigger an update every second
    let mut timer = Timer::syst(cp.SYST, 1.hz(), clocks);
 
    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        block!(timer.wait()).unwrap();
        led.set_high().unwrap();
        block!(timer.wait()).unwrap();
        led.set_low().unwrap();
    }
}
