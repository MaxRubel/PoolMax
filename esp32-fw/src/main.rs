// #![no_std]
// #![no_main]

// use app_core::System;
// use core::sync::atomic::{AtomicBool, Ordering};
// use esp_backtrace as _;
// use esp_println::println;
// use esp32_hal as hal;
// use hal::{
//     gpio::{Event, Gpio0, Input, PullUp},
//     interrupt,
//     peripherals::Peripherals,
//     prelude::*,
// };

// static BUTTON_FLAG: AtomicBool = AtomicBool::new(false);
// static mut SYSTEM: Option<System> = None; // global mutable for ISR access

// #[hal::entry]
// fn main() -> ! {
//     // Take the peripherals
//     let peripherals = Peripherals::take().unwrap();

//     // Setup clocks, IO
//     let system = peripherals.DPORT.split();
//     let clocks = hal::clock::ClockControl::boot_defaults(system.clock_control).freeze();
//     let io = hal::gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

//     // Initialize your system logic
//     unsafe {
//         SYSTEM = Some(System::new());
//     }

//     // Configure the button GPIO
//     let mut button: Gpio0<Input<PullUp>> = io.pins.gpio0.into_pull_up_input();
//     button.listen(Event::FallingEdge); // trigger on press

//     // Enable GPIO interrupt in NVIC
//     unsafe {
//         hal::interrupt::enable(
//             hal::peripherals::Interrupt::GPIO,
//             hal::interrupt::Priority::Priority1,
//         )
//         .unwrap();
//     }

//     println!("ESP32 ready!");

//     loop {
//         // Poll the flag set by ISR
//         if BUTTON_FLAG.swap(false, Ordering::Relaxed) {
//             // Call your app-core logic
//             unsafe {
//                 if let Some(system) = SYSTEM.as_mut() {
//                     system.handle_button_press();
//                     println!("Jets are now on: {}", system.jets_on);
//                 }
//             }
//         }
//     }
// }

// // Interrupt Service Routine
// #[interrupt]
// fn GPIO() {
//     // clear pending events (chip-specific, pseudo-code)
//     hal::gpio::clear_all_pending();

//     // Set the atomic flag
//     BUTTON_FLAG.store(true, Ordering::Relaxed);
// }
